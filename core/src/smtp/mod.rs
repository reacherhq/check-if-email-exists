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
#[cfg(feature = "headless")]
mod headless;
mod http_api;
mod outlook;
mod parser;
mod yahoo;

use std::default::Default;
use std::env;

use async_smtp::EmailAddress;
use hickory_proto::rr::Name;
use serde::{Deserialize, Serialize};

use crate::{
	util::input_output::CheckEmailInput, GmailVerifMethod, HotmailVerifMethod, YahooVerifMethod,
};
use connect::check_smtp_with_retry;
pub use error::*;

use self::{
	gmail::is_gmail,
	outlook::{is_hotmail, is_microsoft365},
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
	let host = host.to_string();
	let to_email_str = to_email.to_string();

	if input.skipped_domains.iter().any(|d| host.contains(d)) {
		return (
			Err(SmtpError::SkippedDomain(format!(
				"Reacher currently cannot verify emails from @{domain}"
			))),
			SmtpDebug {
				verif_method: VerifMethod::Skipped,
			},
		);
	}

	let webdriver_addr = env::var("RCH_WEBDRIVER_ADDR");

	if is_hotmail(&host) {
		match (
			&input.hotmail_verif_method,
			webdriver_addr,
			is_microsoft365(&host),
		) {
			(HotmailVerifMethod::OneDriveApi, _, true) => {
				match outlook::microsoft365::check_microsoft365_api(to_email, input).await {
					Ok(Some(smtp_details)) => {
						return {
							(
								Ok(smtp_details),
								SmtpDebug {
									verif_method: VerifMethod::Api,
								},
							)
						}
					}
					// Continue in the event of an error/ambiguous result.
					Err(err) => {
						return (
							Err(err.into()),
							SmtpDebug {
								verif_method: VerifMethod::Api,
							},
						);
					}
					_ => {}
				}
			}
			#[cfg(feature = "headless")]
			(HotmailVerifMethod::Headless, Ok(a), false) => {
				return (
					outlook::headless::check_password_recovery(to_email.to_string().as_str(), &a)
						.await
						.map_err(|err| err.into()),
					SmtpDebug {
						verif_method: VerifMethod::Headless,
					},
				);
			}
			_ => {}
		};
	} else if is_gmail(&host) {
		if let GmailVerifMethod::Api = &input.gmail_verif_method {
			return (
				gmail::check_gmail(to_email, input)
					.await
					.map_err(|err| err.into()),
				SmtpDebug {
					verif_method: VerifMethod::Api,
				},
			);
		};
	} else if is_yahoo(&host) {
		match (&input.yahoo_verif_method, webdriver_addr) {
			(YahooVerifMethod::Api, _) => {
				return (
					yahoo::check_api(&to_email_str, input)
						.await
						.map_err(|e| e.into()),
					SmtpDebug {
						verif_method: VerifMethod::Api,
					},
				)
			}
			#[cfg(feature = "headless")]
			(YahooVerifMethod::Headless, Ok(a)) => {
				return (
					yahoo::check_headless(&to_email_str, &a)
						.await
						.map_err(|e| e.into()),
					SmtpDebug {
						verif_method: VerifMethod::Headless,
					},
				)
			}
			_ => {}
		};
	}

	(
		check_smtp_with_retry(to_email, &host, port, domain, input, input.retries).await,
		SmtpDebug {
			verif_method: VerifMethod::Smtp(SmtpConnection {
				host,
				port,
				used_proxy: input.proxy.is_some(),
			}),
		},
	)
}

#[cfg(test)]
mod tests {
	use super::{check_smtp, CheckEmailInput, SmtpConnection, SmtpError};
	use async_smtp::{smtp::error::Error, EmailAddress};
	use hickory_proto::rr::Name;
	use std::{str::FromStr, time::Duration};
	use tokio::runtime::Runtime;

	#[test]
	fn should_timeout() {
		let runtime = Runtime::new().unwrap();

		let to_email = EmailAddress::from_str("foo@gmail.com").unwrap();
		let host = Name::from_str("alt4.aspmx.l.google.com.").unwrap();
		let mut input = CheckEmailInput::default();
		input.set_gmail_verif_method(crate::GmailVerifMethod::Smtp);
		input.set_smtp_timeout(Some(Duration::from_millis(1)));

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

	#[test]
	fn should_skip_domains() {
		let runtime = Runtime::new().unwrap();

		let to_email = EmailAddress::from_str("foo@icloud.com").unwrap();
		let host = Name::from_str("mx01.mail.icloud.com.").unwrap();
		let mut input = CheckEmailInput::default();
		input.set_skipped_domains(vec![".mail.icloud.com.".into()]);

		let (res, smtp_debug) =
			runtime.block_on(check_smtp(&to_email, &host, 25, "icloud.com", &input));
		assert_eq!(smtp_debug.verif_method, super::VerifMethod::Skipped);
		match res {
			Err(SmtpError::SkippedDomain(_)) => (),
			r => panic!("{:?}", r),
		}
	}
}
