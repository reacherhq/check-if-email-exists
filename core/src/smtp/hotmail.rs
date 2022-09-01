// check-if-email-exists
// Copyright (C) 2018-2022 Reacher

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{thread::sleep, time::Duration};

use async_smtp::EmailAddress;
use async_std::prelude::FutureExt;
use fantoccini::{
	error::{CmdError, NewSessionError},
	ClientBuilder, Locator,
};
use futures::TryFutureExt;
use serde::Serialize;
use serde_json::Map;

use super::SmtpDetails;
use crate::util::ser_with_display::ser_with_display;
use crate::LOG_TARGET;

#[derive(Debug, Serialize)]
pub enum HotmailError {
	#[serde(serialize_with = "ser_with_display")]
	Cmd(CmdError),
	#[serde(serialize_with = "ser_with_display")]
	NewSession(NewSessionError),
}

impl From<CmdError> for HotmailError {
	fn from(e: CmdError) -> Self {
		Self::Cmd(e)
	}
}

impl From<NewSessionError> for HotmailError {
	fn from(e: NewSessionError) -> Self {
		Self::NewSession(e)
	}
}

/// Check if a Hotmail/Outlook email exists by connecting to the password
/// recovery page https://account.live.com/password/reset using a headless
/// browser. Make sure you have a WebDriver server running locally before
/// running this, or this will error.
pub async fn check_password_recovery(
	to_email: &EmailAddress,
	webdriver: &str,
) -> Result<SmtpDetails, HotmailError> {
	let to_email = to_email.to_string();
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Using Hotmail password recovery in headless navigator",
		to_email,
	);

	// Running in a container, I run into the following error:
	// Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted
	// In searching around I found a few different workarounds:
	// - Enable namespaces: https://github.com/jessfraz/dockerfiles/issues/65#issuecomment-266532289
	// - Run it with a custom seccomp: https://github.com/jessfraz/dockerfiles/issues/65#issuecomment-217214671
	// - Run with --no-sandbox: https://github.com/karma-runner/karma-chrome-launcher/issues/125#issuecomment-312668593
	// For now I went with the --no-sandbox.
	//
	// TODO Look into security implications...
	let mut caps = Map::new();
	let opts = serde_json::json!({
		"args": ["--headless", "--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"],
	});
	caps.insert("goog:chromeOptions".to_string(), opts);

	// Connect to WebDriver instance that is listening on `webdriver`
	let c = ClientBuilder::native()
		.capabilities(caps)
		.connect(webdriver)
		.await?;

	// Navigate to Microsoft password recovery page.
	c.goto("https://account.live.com/password/reset").await?;

	// Wait for network/javascript/dom to make the input-box available
	// and click it.
	let input = c.wait().for_element(Locator::Id("iSigninName")).await?;
	// Sometimes I get "input ... is not reachable by keyboard". Addind this
	// small sleep helps.
	sleep(Duration::from_millis(200));
	input.send_keys(to_email.as_str()).await?;

	// Click on "Next"
	c.find(Locator::Id("resetPwdHipAction"))
		.await?
		.click()
		.await?;

	// "Try entering your Microsoft account again. We don't recognise this one." means the account does not exist.
	let f1 = c
		.wait()
		.for_element(Locator::Id("pMemberNameErr"))
		.and_then(|_| async { Ok(false) });
	// "We need to verify your identity" means that the account exists.
	let f2 = c
		.wait()
		.for_element(Locator::Id("iSelectProofTitle"))
		.and_then(|_| async { Ok(true) });
	let is_deliverable = f1.try_race(f2).await?;

	if is_deliverable {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Did not find error message in password recovery, email exists",
			to_email,
		);
	} else {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Found error message in password recovery, email does not exist",
			to_email,
		);
	}

	c.close().await?;

	Ok(SmtpDetails {
		can_connect_smtp: true,
		has_full_inbox: false,
		is_catch_all: false,
		is_deliverable,
		is_disabled: false,
	})
}

#[cfg(test)]
mod tests {
	use super::check_password_recovery;
	use async_smtp::EmailAddress;
	use async_std::prelude::FutureExt;
	use std::str::FromStr;

	// Ignoring this test as it requires a local process of WebDriver running on
	// "http://localhost:4444". To debug the headless password recovery page,
	// run chromedriver and remove the "#[ignore]".
	// Also see: https://github.com/jonhoo/fantoccini
	#[tokio::test]
	#[ignore]
	async fn test_hotmail_address() {
		// This email does not exist.
		let email = EmailAddress::from_str("test42134@hotmail.com").unwrap();
		// Run 10 headless sessions with the above fake email (not deliverable).
		// It should not error.
		for _ in 0..10 {
			let res = check_password_recovery(&email, "http://localhost:4444")
				.await
				.unwrap();
			assert!(!res.is_deliverable)
		}

		// This email does exist.
		let email = EmailAddress::from_str("test@hotmail.com").unwrap();
		// Run 10 headless sessions with the above fake email (not deliverable).
		// It should not error.
		for _ in 0..10 {
			let res = check_password_recovery(&email, "http://localhost:4444")
				.await
				.unwrap();
			assert!(res.is_deliverable)
		}
	}

	// This test tests that we can run 2 instances of check_password_recovery.
	// This will only work with chromedriver (which supports parallel cleints),
	// but will fail with geckodriver.
	// ref: https://github.com/jonhoo/fantoccini/issues/111#issuecomment-727650629
	#[tokio::test]
	#[ignore]
	async fn test_parallel() {
		// This email does not exist.
		let email = EmailAddress::from_str("foo@bar.baz").unwrap();

		let f1 = check_password_recovery(&email, "http://localhost:4444");
		let f2 = check_password_recovery(&email, "http://localhost:4444");

		let f = f1.try_join(f2).await;
		assert!(f.is_ok(), "{:?}", f);
	}
}
