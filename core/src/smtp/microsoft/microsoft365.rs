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
use reqwest::Error as ReqwestError;
use serde::Serialize;

use crate::{
	smtp::{http_api::create_client, SmtpDetails},
	util::ser_with_display::ser_with_display,
	CheckEmailInput, LOG_TARGET,
};

#[derive(Debug, Serialize)]
pub enum Microsoft365Error {
	#[serde(serialize_with = "ser_with_display")]
	ReqwestError(ReqwestError),
}

impl From<ReqwestError> for Microsoft365Error {
	fn from(error: ReqwestError) -> Self {
		Microsoft365Error::ReqwestError(error)
	}
}

/// Convert an email address to its corresponding OneDrive URL.
fn get_onedrive_url(email_address: &str) -> String {
	let (username, domain) = email_address
		.split_once('@')
		.expect("Email address syntax already validated.");
	let (tenant, _) = domain
		.split_once('.')
		.expect("Email domain syntax already validated.");

	format!(
		"https://{}-my.sharepoint.com/personal/{}_{}/_layouts/15/onedrive.aspx",
		tenant,
		username.replace('.', "_"),
		domain.replace('.', "_"),
	)
}

/// Use a HTTP request to verify if an Microsoft 365 email address exists.
///
/// See
/// [this article](<https://www.trustedsec.com/blog/achieving-passive-user-enumeration-with-onedrive/>)
/// for details on the underlying principles.
///
/// Note that a positive response from this function is (at present) considered
/// a reliable indicator that an email-address is valid. However, a negative
/// response is ambigious: the email address may or may not be valid but this
/// cannot be determined by the method outlined here.
pub async fn check_microsoft365_api(
	to_email: &EmailAddress,
	input: &CheckEmailInput,
) -> Result<Option<SmtpDetails>, Microsoft365Error> {
	let url = get_onedrive_url(to_email.as_ref());

	let response = create_client(input, "microsoft365")?
		.head(url)
		.send()
		.await?;

	log::debug!(
		target: LOG_TARGET,
		"[email={}] microsoft365 response: {:?}",
		to_email,
		response
	);

	if response.status() == 403 {
		Ok(Some(SmtpDetails {
			can_connect_smtp: true,
			is_deliverable: true,
			..Default::default()
		}))
	} else {
		Ok(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_onedrive_url() {
		let email_address = "lightmand@acmecomputercompany.com";
		let expected = "https://acmecomputercompany-my.sharepoint.com/personal/lightmand_acmecomputercompany_com/_layouts/15/onedrive.aspx";

		assert_eq!(expected, get_onedrive_url(email_address));
	}
}
