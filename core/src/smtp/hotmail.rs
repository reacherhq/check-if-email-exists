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

use std::fs;
use std::time::Duration;

use async_smtp::EmailAddress;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use serde::Serialize;

use super::SmtpDetails;
use crate::util::ser_with_display::ser_with_display;
use crate::LOG_TARGET;

#[derive(Debug, Serialize)]
pub struct HotmailError {
	#[serde(serialize_with = "ser_with_display")]
	err: anyhow::Error,
}

impl From<anyhow::Error> for HotmailError {
	fn from(err: anyhow::Error) -> Self {
		Self { err }
	}
}

/// Check if a Hotmail/Outlook email exists by connecting to the password
/// recovery page https://account.live.com/password/reset using a headless
/// Chrome browser. Make sure you have Chrome/Chromium installed locally before
/// running this, or this will error.
pub fn check_password_recovery(to_email: &EmailAddress) -> Result<SmtpDetails, HotmailError> {
	let mut res = run_headless(to_email, false);

	// In some rare cases, `run_headless` errors, with the following message:
	// "Scrolling element into view failed: Node is detached from document"
	// In those cases, re-run max 2 times more.
	for _ in 0..2 {
		if res.is_ok() {
			return res;
		}

		res = run_headless(to_email, false);
	}

	res
}

/// Launch a headless navigator to perform email password recovery. Optionally
/// takes a screenshot of the last page, and saves it to disk, by setting
/// `save_jpg` to true; only use this option while debugging.
fn run_headless(to_email: &EmailAddress, save_jpg: bool) -> Result<SmtpDetails, HotmailError> {
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Using Hotmail password recovery in headless navigator",
		to_email,
	);
	let options = LaunchOptionsBuilder::default()
		.window_size(Some((1800, 1500)))
		.sandbox(false)
		.build()
		.unwrap();
	let browser = Browser::new(options)?;
	let tab = browser.wait_for_initial_tab()?;
	let to_email = to_email.to_string();

	// Navigate to Microsoft password recovery page.
	tab.navigate_to("https://account.live.com/password/reset")?;

	// Wait for network/javascript/dom to make the input-box available
	// and click it.
	tab.wait_for_element("input#iSigninName")?.click()?;

	// Type in a query and press `Enter`
	tab.type_str(to_email.as_str())?.press_key("Enter")?;

	// We should end up on the WebKit-page once navigated
	tab.wait_until_navigated()?;

	// Somehow, empirically, it works better by waiting first for #signinNameSection,
	// then waiting for its child #pMemberNameErr.
	tab.wait_for_element_with_custom_timeout("#signinNameSection", Duration::from_secs(2))?;
	let account_does_not_exist = tab
		.wait_for_element_with_custom_timeout("#pMemberNameErr", Duration::from_secs(1))
		.is_ok();

	if save_jpg {
		let jpeg_data =
			tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)?;
		fs::write("hotmail.jpeg", &jpeg_data)
			.expect("Safe to unwrap, as save_jpg should only be used for debugging purposes.");
	}

	if account_does_not_exist {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Found error message in password recovery, email does not exist",
			to_email,
		);
	} else {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Did not find error message in password recovery, email exists",
			to_email,
		);
	}

	Ok(SmtpDetails {
		can_connect_smtp: true,
		has_full_inbox: false,
		is_catch_all: false,
		is_deliverable: !account_does_not_exist,
		is_disabled: false,
	})
}

#[cfg(test)]
mod tests {
	use super::run_headless;
	use async_smtp::EmailAddress;
	use std::str::FromStr;

	// Ignoring this test as it requires a local instance of Chromium. To debug
	// the headless password recovery page, simply remove the "#[ignore]".
	#[test]
	#[ignore]
	fn test_hotmail_address() {
		let email = EmailAddress::from_str("test42134@hotmail.com").unwrap();
		// Run 10 headless sessions with the above fake email (not deliverable).
		// It should not error.
		for _ in 0..10 {
			let res = run_headless(&email, true).unwrap();
			assert!(!res.is_deliverable)
		}
	}
}
