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

use std::{thread::sleep, time::Duration};

use async_std::prelude::FutureExt;
use fantoccini::Locator;
use futures::TryFutureExt;

use crate::smtp::headless::{create_headless_client, HeadlessError};
use crate::{smtp::SmtpDetails, LOG_TARGET};

/// Check if a Hotmail/Outlook email exists by connecting to the password
/// recovery page https://account.live.com/password/reset using a headless
/// browser. Make sure you have a WebDriver server running locally before
/// running this, or this will error.
pub async fn check_headless(to_email: &str, webdriver: &str) -> Result<SmtpDetails, HeadlessError> {
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Using Yahoo password recovery in headless navigator",
		to_email,
	);

	let c = create_headless_client(webdriver).await?;

	// Navigate to Microsoft password recovery page.
	c.goto("https://login.yahoo.com/forgot").await?;

	// Wait for network/javascript/dom to make the input-box available
	// and click it.
	let input = c.wait().for_element(Locator::Id("username")).await?;
	// Sometimes I get "input ... is not reachable by keyboard". Addind this
	// small sleep helps.
	sleep(Duration::from_millis(200));
	input.send_keys(to_email).await?;

	// Click on "Continue"
	c.find(Locator::Css("button[name=\"verifyYid\"]"))
		.await?
		.click()
		.await?;

	// Depending on what answers we have on the account recovery page, return
	// the relevant (is_deliverable, is_disabled) tuple.

	// "Sorry, we don't recognise that email address or phone number".
	let f1 = c
		.wait()
		.for_element(Locator::Css(".error-msg"))
		.and_then(|_| async { Ok((false, false)) });
	// "This account has been deactivated due to inactivity, but we would love to welcome you back!"
	let f2 = c
		.wait()
		.for_element(Locator::Css(".ctx-account_is_locked"))
		.and_then(|_| async { Ok((false, true)) });
	// Recaptcha
	let f3 = c
		.wait()
		.for_element(Locator::Css(".recaptcha-challenge"))
		.and_then(|_| async { Ok((true, false)) });
	// "Enter verification code sent to your email address"
	let f4 = c
		.wait()
		.for_element(Locator::Id("email-verify-challenge"))
		.and_then(|_| async { Ok((true, false)) });
	// "Select an option to sign in to your account"
	let f5 = c
		.wait()
		.for_element(Locator::Id("challenge-selector-challenge"))
		.and_then(|_| async { Ok((true, false)) });

	let (is_deliverable, is_disabled) = f1
		.try_race(f2)
		.try_race(f3)
		.try_race(f4)
		.try_race(f5)
		.await?;

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
		is_disabled,
	})
}

#[cfg(test)]
mod tests {
	use super::check_headless;

	// Ignoring this test as it requires a local process of WebDriver running on
	// "http://localhost:9515". To debug the headless password recovery page,
	// run chromedriver and remove the "#[ignore]".
	// Also see: https://github.com/jonhoo/fantoccini
	#[tokio::test]
	#[ignore = "Run a webdriver server locally to test this"]
	async fn test_yahoo_address() {
		// Run 5 headless sessions with the below dummy emails.
		for _ in 0..5 {
			// Email does not exist.
			let res = check_headless("test42134@yahoo.com", "http://localhost:9515")
				.await
				.unwrap();
			assert!(!res.is_deliverable);

			// Disabled email.
			let res = check_headless("amaury@yahoo.com", "http://localhost:9515")
				.await
				.unwrap();
			assert!(!res.is_deliverable);
			assert!(res.is_disabled);

			// OK email.
			let res = check_headless("test2@yahoo.com", "http://localhost:9515")
				.await
				.unwrap();
			assert!(res.is_deliverable);
			assert!(!res.is_disabled);
		}
	}
}
