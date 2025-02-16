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
pub mod verif_method;
mod yahoo;

use crate::util::input_output::CheckEmailInput;
use crate::EmailAddress;
use connect::check_smtp_with_retry;
use hickory_proto::rr::Name;
use serde::{Deserialize, Serialize};
use std::default::Default;
use verif_method::{
	EmailProvider, EverythingElseVerifMethod, GmailVerifMethod, HotmailB2BVerifMethod,
	HotmailB2CVerifMethod, VerifMethodSmtp, VerifMethodSmtpConfig, YahooVerifMethod,
};

pub use crate::mx::{is_gmail, is_hotmail, is_hotmail_b2b, is_hotmail_b2c, is_yahoo};
pub use error::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct SmtpDebugVerifMethodSmtp {
	/// The host we connected to via SMTP.
	pub host: String,
	/// The proxy used for the SMTP connection.
	pub verif_method: VerifMethodSmtpConfig,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SmtpDebugVerifMethod {
	/// Email verification was done via SMTP.
	Smtp(SmtpDebugVerifMethodSmtp),
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

/// Debug information on how the SMTP verification went.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SmtpDebug {
	/// The verification method used for the email.
	pub verif_method: SmtpDebugVerifMethod,
}

/// Get all email details we can from one single `EmailAddress`, without
/// retries.
pub async fn check_smtp(
	to_email: &EmailAddress,
	host: &Name,
	domain: &str,
	input: &CheckEmailInput,
) -> (Result<SmtpDetails, SmtpError>, SmtpDebug) {
	let host_str = host.to_string();
	let to_email_str = to_email.to_string();
	let email_provider = EmailProvider::from_mx_host(&host_str);

	// Handle all non-SMTP verifications first, and return early. For the rest,
	// we'll use SMTP, and return the config.
	let smtp_verif_method_config = match &email_provider {
		EmailProvider::HotmailB2C => match &input.verif_method.hotmailb2c {
			HotmailB2CVerifMethod::Headless => {
				return (
					outlook::headless::check_password_recovery(
						&to_email_str,
						&input.webdriver_addr,
						&input.webdriver_config,
					)
					.await
					.map_err(Into::into),
					SmtpDebug {
						verif_method: SmtpDebugVerifMethod::Headless,
					},
				);
			}
			HotmailB2CVerifMethod::Smtp(c) => c,
		},
		EmailProvider::Yahoo => match &input.verif_method.yahoo {
			YahooVerifMethod::Api => {
				return (
					yahoo::check_api(&to_email_str, input)
						.await
						.map_err(Into::into),
					SmtpDebug {
						verif_method: SmtpDebugVerifMethod::Api,
					},
				);
			}
			YahooVerifMethod::Headless => {
				return (
					yahoo::check_headless(
						&to_email_str,
						&input.webdriver_addr,
						&input.webdriver_config,
					)
					.await
					.map_err(Into::into),
					SmtpDebug {
						verif_method: SmtpDebugVerifMethod::Headless,
					},
				);
			}
			YahooVerifMethod::Smtp(c) => c,
		},
		EmailProvider::Gmail => match &input.verif_method.gmail {
			GmailVerifMethod::Smtp(c) => c,
		},
		EmailProvider::HotmailB2B => match &input.verif_method.hotmailb2b {
			HotmailB2BVerifMethod::Smtp(c) => c,
		},
		EmailProvider::Mimecast => match &input.verif_method.mimecast {
			verif_method::MimecastVerifMethod::Smtp(c) => c,
		},
		EmailProvider::Proofpoint => match &input.verif_method.proofpoint {
			verif_method::ProofpointVerifMethod::Smtp(c) => c,
		},
		EmailProvider::EverythingElse => match &input.verif_method.everything_else {
			EverythingElseVerifMethod::Smtp(c) => c,
		},
	}
	.clone();

	// TODO: There's surely a way to not clone here.
	let verif_method = VerifMethodSmtp::new(
		smtp_verif_method_config.clone(),
		input.verif_method.get_proxy(email_provider).cloned(),
	);

	(
		check_smtp_with_retry(
			to_email,
			&host_str,
			domain,
			&verif_method,
			verif_method.config.retries,
		)
		.await,
		SmtpDebug {
			verif_method: SmtpDebugVerifMethod::Smtp(SmtpDebugVerifMethodSmtp {
				host: host_str,
				verif_method: smtp_verif_method_config,
			}),
		},
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::smtp::verif_method::GmailVerifMethod;
	use crate::smtp::verif_method::VerifMethod;
	use crate::smtp::verif_method::VerifMethodSmtpConfig;
	use crate::CheckEmailInputBuilder;
	use crate::EmailAddress;
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
			.verif_method(VerifMethod {
				gmail: GmailVerifMethod::Smtp(VerifMethodSmtpConfig {
					smtp_timeout: Some(Duration::from_millis(1)),
					retries: 1,
					..Default::default()
				}),
				..Default::default()
			})
			.build()
			.unwrap();

		let (res, smtp_debug) = runtime.block_on(check_smtp(&to_email, &host, "gmail.com", &input));
		match smtp_debug.verif_method {
			SmtpDebugVerifMethod::Smtp(SmtpDebugVerifMethodSmtp { host, verif_method }) => {
				assert_eq!(host, "alt4.aspmx.l.google.com.");
				assert_eq!(verif_method.smtp_port, 25);
				assert_eq!(verif_method.smtp_timeout, Some(Duration::from_millis(1)));
				assert_eq!(verif_method.retries, 1);
				assert_eq!(verif_method.proxy, None);
			}
			_ => panic!("Expected SmtpDebugVerifMethod::Smtp"),
		}

		match res {
			Err(SmtpError::Timeout(_)) => (), // ErrorKind == Timeout
			_ => panic!("check_smtp did not time out"),
		}
	}
}
