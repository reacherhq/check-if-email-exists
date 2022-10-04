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

use super::SmtpDetails;
use crate::util::{
	constants::LOG_TARGET, input_output::CheckEmailInput, ser_with_display::ser_with_display,
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
		write!(f, "{:?}", self)
	}
}

impl From<ReqwestError> for GmailError {
	fn from(error: ReqwestError) -> Self {
		GmailError::ReqwestError(error)
	}
}

/// Helper function to create a reqwest client, with optional proxy.
fn create_client(input: &CheckEmailInput) -> Result<reqwest::Client, ReqwestError> {
	if let Some(proxy) = &input.proxy {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Using proxy socks://{}:{} for gmail API",
			input.to_email,
			proxy.host,
			proxy.port
		);

		let proxy = reqwest::Proxy::all(&format!("socks5://{}:{}", proxy.host, proxy.port))?;
		reqwest::Client::builder().proxy(proxy).build()
	} else {
		Ok(reqwest::Client::new())
	}
}

/// Use HTTP request to verify if a Gmail email address exists.
/// See: <https://blog.0day.rocks/abusing-gmail-to-get-previously-unlisted-e-mail-addresses-41544b62b2>
pub async fn check_gmail(
	to_email: &EmailAddress,
	input: &CheckEmailInput,
) -> Result<SmtpDetails, GmailError> {
	let response = create_client(input)?
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
