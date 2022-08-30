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

use async_smtp::EmailAddress;
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
	let is_error = tab.find_element("#pMemberNameErr").is_ok();

	if is_error {
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
		is_deliverable: !is_error,
		is_disabled: false,
	})
}
