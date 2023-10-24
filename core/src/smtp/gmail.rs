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

use super::SmtpDetails;
use crate::{
	smtp::http_api::create_client,
	util::{
		constants::LOG_TARGET, input_output::CheckEmailInput, ser_with_display::ser_with_display,
	},
};
use async_smtp::EmailAddress;
use reqwest::Error as ReqwestError;
use serde::Serialize;
use std::fmt;

const GLXU_PAGE: &str = "https://mail.google.com/mail/gxlu";

/// Possible errors when checking Gmail email addresses.
#[derive(Debug, Serialize)]
pub enum GmailError {
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	ReqwestError(ReqwestError),
}

impl fmt::Display for GmailError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:?}")
	}
}

impl From<ReqwestError> for GmailError {
	fn from(error: ReqwestError) -> Self {
		GmailError::ReqwestError(error)
	}
}

/// Use HTTP request to verify if a Gmail email address exists.
/// See: <https://blog.0day.rocks/abusing-gmail-to-get-previously-unlisted-e-mail-addresses-41544b62b2>
pub async fn check_gmail(
	to_email: &EmailAddress,
	input: &CheckEmailInput,
) -> Result<SmtpDetails, GmailError> {
	let response = create_client(input, "gmail")?
		.head(GLXU_PAGE)
		.query(&[("email", to_email)])
		.send()
		.await?;

	let email_exists = response.headers().contains_key("Set-Cookie");

	log::debug!(
		target: LOG_TARGET,
		"[email={}] gmail response: {:?}",
		to_email,
		response
	);

	Ok(SmtpDetails {
		can_connect_smtp: true,
		is_deliverable: email_exists,
		..Default::default()
	})
}

/// Check if the MX host is from Google, i.e. either a @gmail.com address, or
/// a Google Suite email.
pub fn is_gmail(host: &str) -> bool {
	host.to_lowercase().ends_with(".google.com.")
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use super::*;

	#[tokio::test]
	async fn should_return_is_deliverable_true() {
		let to_email = EmailAddress::from_str("someone@gmail.com").unwrap();
		let input = CheckEmailInput::new("someone@gmail.com".to_owned());

		let smtp_details = check_gmail(&to_email, &input).await;

		assert!(smtp_details.is_ok());
		assert!(smtp_details.unwrap().is_deliverable);
	}
}
