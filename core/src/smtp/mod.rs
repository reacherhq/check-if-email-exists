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

mod connect;
mod error;
mod gmail;

mod headless;
mod http_api;
mod outlook;
mod parser;
mod yahoo;

use std::default::Default;

use async_smtp::EmailAddress;
use hickory_proto::rr::Name;
use serde::{Deserialize, Serialize};

use crate::{
	util::input_output::CheckEmailInput, GmailVerifMethod, HotmailB2CVerifMethod, YahooVerifMethod,
};
use connect::check_smtp_with_retry;
pub use error::*;

pub use self::{
	gmail::is_gmail,
	outlook::{is_hotmail, is_hotmail_b2b, is_hotmail_b2c},
	yahoo::is_yahoo,
};

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SmtpConnection {
	/// The host we connected to via SMTP.
	pub host: String,
	/// The port we connected to via SMTP.
	pub port: u16,
	/// Whether we used a proxy for the SMTP connection.
	pub used_proxy: bool,
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum VerifMethod {
	/// Email verification was done via SMTP.
	Smtp(SmtpConnection),
	/// Email verification was done via an HTTP API.
	Api,
	/// Email verification was done via a headless browser.
	Headless,
	/// Email verification was skipped.
	#[default]
	Skipped,
}

/// Details that we gathered from connecting to this email via SMTP
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SmtpDetails {
	/// Are we able to connect to the SMTP server?
	pub can_connect_smtp: bool,
	/// Is this email account's inbox full?
	pub has_full_inbox: bool,
	/// Does this domain have a catch-all email address?
	pub is_catch_all: bool,
	/// Can we send an email to this address?
	pub is_deliverable: bool,
	/// Is the email blocked or disabled by the provider?
	pub is_disabled: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SmtpDebug {
	/// The verification method used for the email.
	pub verif_method: VerifMethod,
}

/// Get all email details we can from one single `EmailAddress`, without
/// retries.
pub async fn check_smtp(
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
) -> (Result<SmtpDetails, SmtpError>, SmtpDebug) {
	let host_str = host.to_string();
	let to_email_str = to_email.to_string();

	if is_hotmail_b2c(&host_str) {
		if let HotmailB2CVerifMethod::Headless = &input.hotmailb2c_verif_method {
			return (
				outlook::headless::check_password_recovery(&to_email_str, &input.webdriver_addr)
					.await
					.map_err(Into::into),
				SmtpDebug {
					verif_method: VerifMethod::Headless,
				},
			);
		}
	} else if is_gmail(&host_str) {
		if let GmailVerifMethod::Api = &input.gmail_verif_method {
			return (
				gmail::check_gmail_via_api(to_email, input)
					.await
					.map_err(Into::into),
				SmtpDebug {
					verif_method: VerifMethod::Api,
				},
			);
		}
	} else if is_yahoo(&host_str) {
		match &input.yahoo_verif_method {
			YahooVerifMethod::Api => {
				return (
					yahoo::check_api(&to_email_str, input)
						.await
						.map_err(Into::into),
					SmtpDebug {
						verif_method: VerifMethod::Api,
					},
				);
			}
			YahooVerifMethod::Headless => {
				return (
					yahoo::check_headless(&to_email_str, &input.webdriver_addr)
						.await
						.map_err(Into::into),
					SmtpDebug {
						verif_method: VerifMethod::Headless,
					},
				);
			}
			_ => {} // For everything else, we use SMTP
		}
	}

	(
		check_smtp_with_retry(to_email, &host_str, port, domain, input, input.retries).await,
		SmtpDebug {
			verif_method: VerifMethod::Smtp(SmtpConnection {
				host: host_str,
				port,
				used_proxy: input.proxy.is_some(),
			}),
		},
	)
}

#[cfg(test)]
mod tests {
	use super::{check_smtp, SmtpConnection, SmtpError};
	use crate::CheckEmailInputBuilder;
	use async_smtp::{smtp::error::Error, EmailAddress};
	use hickory_proto::rr::Name;
	use std::{str::FromStr, time::Duration};
	use tokio::runtime::Runtime;

	#[test]
	fn should_timeout() {
		let runtime = Runtime::new().unwrap();

		let to_email = EmailAddress::from_str("foo@gmail.com").unwrap();
		let host = Name::from_str("alt4.aspmx.l.google.com.").unwrap();
		let input = CheckEmailInputBuilder::default()
			.to_email("foo@gmail.com".into())
			.gmail_verif_method(crate::GmailVerifMethod::Smtp)
			.smtp_timeout(Some(Duration::from_millis(1)))
			.build()
			.unwrap();

		let (res, smtp_debug) =
			runtime.block_on(check_smtp(&to_email, &host, 25, "gmail.com", &input));
		assert_eq!(
			smtp_debug.verif_method,
			super::VerifMethod::Smtp(SmtpConnection {
				host: host.to_string(),
				port: 25,
				used_proxy: input.proxy.is_some(),
			})
		);
		match res {
			Err(SmtpError::SmtpError(Error::Io(_))) => (), // ErrorKind == Timeout
			_ => panic!("check_smtp did not time out"),
		}
	}
}
