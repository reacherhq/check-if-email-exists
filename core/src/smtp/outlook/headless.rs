// check-if-email-exists
// Copyright (C) 2018-2023 Reacher

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

use std::{pin::Pin, thread::sleep, time::Duration};

use fantoccini::{error::CmdError, Locator};
use futures::{future::select_ok, Future, TryFutureExt};

use crate::{
	smtp::{
		headless::{create_headless_client, HeadlessError},
		SmtpDetails,
	},
	LOG_TARGET,
};

/// Check if a Hotmail/Outlook email exists by connecting to the password
/// recovery page https://account.live.com/password/reset using a headless
/// browser. Make sure you have a WebDriver server running locally before
/// running this, or this will error.
pub async fn check_password_recovery(
	to_email: &str,
	webdriver: &str,
) -> Result<SmtpDetails, HeadlessError> {
	let to_email = to_email.to_string();
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Using Hotmail password recovery in headless navigator",
		to_email,
	);

	let c = create_headless_client(webdriver).await?;

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
	// "Try entering your Microsoft account again. We don't recognise this one."
	let f2 = c
		.wait()
		.for_element(Locator::Id("iSigninNameError"))
		.and_then(|_| async { Ok(false) });
	// "We need to verify your identity" means that the account exists.
	let f3 = c
		.wait()
		.for_element(Locator::Id("iSelectProofTitle"))
		.and_then(|_| async { Ok(true) });
	// "Enter the code generated by your authenticator app..."
	let f4 = c
		.wait()
		.for_element(Locator::Id("iEnterVerification"))
		.and_then(|_| async { Ok(true) });

	let vec = vec![
		Box::pin(f1) as Pin<Box<dyn Future<Output = Result<bool, CmdError>> + Send>>,
		Box::pin(f2),
		Box::pin(f3),
		Box::pin(f4),
	];
	let (is_deliverable, _) = select_ok(vec).await?;

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
	use futures::future::join;

	// Ignoring this test as it requires a local process of WebDriver running on
	// "http://localhost:9515". To debug the headless password recovery page,
	// run chromedriver and remove the "#[ignore]".
	// Also see: https://github.com/jonhoo/fantoccini
	#[tokio::test]
	#[ignore = "Run a webdriver server locally to test this"]
	async fn test_hotmail_address() {
		// Run 10 headless sessions with dummy emails.
		// It should not error.
		for _ in 0..10 {
			// This email does not exist.
			let res = check_password_recovery("test42134@hotmail.com", "http://localhost:9515")
				.await
				.unwrap();
			assert!(!res.is_deliverable);

			// This email does exist.
			let res = check_password_recovery("test@hotmail.com", "http://localhost:9515")
				.await
				.unwrap();
			assert!(res.is_deliverable);
		}
	}

	// This test tests that we can run 2 instances of check_password_recovery.
	// This will only work with chromedriver (which supports parallel cleints),
	// but will fail with geckodriver.
	// ref: https://github.com/jonhoo/fantoccini/issues/111#issuecomment-727650629
	#[tokio::test]
	#[ignore = "Run a webdriver server locally to test this"]
	async fn test_parallel() {
		// This email does not exist.
		let f1 = check_password_recovery("foo@bar.baz", "http://localhost:9515");
		let f2 = check_password_recovery("foo@bar.baz", "http://localhost:9515");

		let f = join(f1, f2).await;
		assert!(f.0.is_ok(), "{:?}", f);
	}
}
