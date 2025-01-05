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

use std::pin::Pin;
use std::vec;
use std::{thread::sleep, time::Duration};

use fantoccini::error::CmdError;
use fantoccini::Locator;
use futures::future::select_ok;
use futures::{Future, TryFutureExt};

use crate::smtp::headless::{create_headless_client, HeadlessError};
use crate::WebdriverConfig;
use crate::{smtp::SmtpDetails, LOG_TARGET};

/// Check if a Hotmail/Outlook email exists by connecting to the password
/// recovery page https://account.live.com/password/reset using a headless
/// browser. Make sure you have a WebDriver server running locally before
/// running this, or this will error.
pub async fn check_headless(
	to_email: &str,
	webdriver: &str,
	webdriver_config: &WebdriverConfig,
) -> Result<SmtpDetails, HeadlessError> {
	let mut attempts = 0;
	let max_attempts = 3;
	let mut last_error = None;

	while attempts < max_attempts {
		attempts += 1;
		tracing::debug!(
			target: LOG_TARGET,
			email=%to_email,
			attempt=%attempts,
			"Using Yahoo password recovery in headless navigator"
		);

		match check_headless_inner(to_email, webdriver, webdriver_config).await {
			Ok(result) => return Ok(result),
			Err(e) => {
				last_error = Some(e);
				if attempts < max_attempts {
					sleep(Duration::from_secs(1));
				}
			}
		}
	}

	Err(last_error.unwrap())
}

async fn check_headless_inner(
	to_email: &str,
	webdriver: &str,
	webdriver_config: &WebdriverConfig,
) -> Result<SmtpDetails, HeadlessError> {
	let c = create_headless_client(webdriver, webdriver_config).await?;

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

	let vec = vec![
		Box::pin(f1) as Pin<Box<dyn Future<Output = Result<(bool, bool), CmdError>> + Send>>,
		Box::pin(f2),
		Box::pin(f3),
		Box::pin(f4),
		Box::pin(f5),
	];
	let ((is_deliverable, is_disabled), _) = select_ok(vec).await?;

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
	use crate::{initialize_crypto_provider, WebdriverConfig};

	use super::check_headless;

	// Ignoring this test as it requires a local process of WebDriver running on
	// "http://localhost:9515". To debug the headless password recovery page,
	// run chromedriver and remove the "#[ignore]".
	// Also see: https://github.com/jonhoo/fantoccini
	#[tokio::test]
	#[ignore = "Run a webdriver server on port 9515 locally to test this"]
	async fn test_yahoo_address() {
		initialize_crypto_provider();
		// Run 5 headless sessions with the below dummy emails.
		for _ in 0..5 {
			// Email does not exist.
			let res = check_headless(
				"test42134@yahoo.com",
				"http://localhost:9515",
				&WebdriverConfig::default(),
			)
			.await
			.unwrap();
			assert!(!res.is_deliverable);

			// Disabled email.
			let res = check_headless(
				"amaury@yahoo.com",
				"http://localhost:9515",
				&WebdriverConfig::default(),
			)
			.await
			.unwrap();
			assert!(!res.is_deliverable);
			assert!(res.is_disabled);

			// OK email.
			let res = check_headless(
				"test2@yahoo.com",
				"http://localhost:9515",
				&WebdriverConfig::default(),
			)
			.await
			.unwrap();
			assert!(res.is_deliverable);
			assert!(!res.is_disabled);
		}
	}
}
