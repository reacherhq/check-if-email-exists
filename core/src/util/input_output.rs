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

use crate::misc::{MiscDetails, MiscError};
use crate::mx::{MxDetails, MxError};
use crate::smtp::verif_method::VerifMethod;
use crate::smtp::{SmtpDebug, SmtpDetails, SmtpError, SmtpErrorDesc};
use crate::syntax::SyntaxDetails;
use crate::util::ser_with_display::ser_with_display;
use async_smtp::EmailAddress as AsyncSmtpEmailAddress;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

/// Wrapper around the `EmailAddress` from `async_smtp` to allow for
/// serialization and deserialization.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EmailAddress(AsyncSmtpEmailAddress);

impl Serialize for EmailAddress {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		ser_with_display(&self.0, serializer)
	}
}

impl<'de> Deserialize<'de> for EmailAddress {
	fn deserialize<D>(deserializer: D) -> Result<EmailAddress, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(EmailAddress(
			AsyncSmtpEmailAddress::from_str(&s).map_err(serde::de::Error::custom)?,
		))
	}
}

impl Display for EmailAddress {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl FromStr for EmailAddress {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(EmailAddress(AsyncSmtpEmailAddress::from_str(s)?))
	}
}

impl EmailAddress {
	pub fn new(email: String) -> Result<Self, anyhow::Error> {
		Ok(EmailAddress(AsyncSmtpEmailAddress::new(email)?))
	}

	pub fn into_inner(self) -> AsyncSmtpEmailAddress {
		self.0
	}
}

impl AsRef<AsyncSmtpEmailAddress> for EmailAddress {
	fn as_ref(&self) -> &AsyncSmtpEmailAddress {
		&self.0
	}
}

impl AsRef<str> for EmailAddress {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

/// Perform the email verification via a specified proxy. The usage of a proxy
/// is optional.
#[derive(Debug, Default, Clone, Deserialize, PartialEq, Serialize)]
pub struct CheckEmailInputProxy {
	/// Use the specified SOCKS5 proxy host to perform email verification.
	pub host: String,
	/// Use the specified SOCKS5 proxy port to perform email verification.
	pub port: u16,
	/// Username to pass to proxy authentication.
	pub username: Option<String>,
	/// Password to pass to proxy authentication.
	pub password: Option<String>,
	/// The timeout for the connection to the proxy, in ms.
	pub timeout_ms: Option<u64>,
}

/// Builder pattern for the input argument into the main `email_exists`
/// function.
#[derive(Builder, Debug, Clone, Deserialize, Serialize)]
#[builder(default)]
pub struct CheckEmailInput {
	/// The email to validate.
	pub to_email: String,

	pub verif_method: VerifMethod,

	/// Whether to check if a gravatar image is existing for the given email.
	/// Adds a bit of latency to the verification process.
	///
	/// Defaults to false.
	pub check_gravatar: bool,
	/// Check if a the email address is present in HaveIBeenPwned API.
	/// If the api_key is filled, HaveIBeenPwned API is checked
	pub haveibeenpwned_api_key: Option<String>,

	/// The WebDriver address to use for headless verifications.
	///
	/// Defaults to http://localhost:9515.
	pub webdriver_addr: String,
	/// The WebDriver configuration to use for headless verifications.
	///
	/// Defaults to the default WebdriverConfig.
	pub webdriver_config: WebdriverConfig,
	/// Identifier for the service currently running Reacher. We recommend
	/// setting this to an unique identifier of the server where Reacher is
	/// installed on.
	///
	/// Defaults to "backend-dev".
	pub backend_name: String,
	/// Sentry DSN to send errors to Sentry.
	///
	/// Defaults to None.
	pub sentry_dsn: Option<String>,
}

impl Default for CheckEmailInput {
	fn default() -> Self {
		CheckEmailInput {
			to_email: "".into(),
			verif_method: VerifMethod::default(),
			check_gravatar: false,
			haveibeenpwned_api_key: None,
			webdriver_addr: "http://localhost:9515".into(),
			webdriver_config: WebdriverConfig::default(),
			backend_name: "backend-dev".into(),
			sentry_dsn: None,
		}
	}
}

/// An enum to describe how confident we are that the recipient address is
/// real.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reachable {
	/// The email is safe to send.
	Safe,
	/// The email address appears to exist, but has quality issues that may
	/// result in low engagement or a bounce. Emails are classified as risky
	/// when one of the following happens:
	/// - catch-all email,
	/// - disposable email,
	/// - role-based address,
	/// - full inbox.
	Risky,
	/// Emails that don't exist or are syntactically incorrect. Do not send to
	/// these emails.
	Invalid,
	/// We're unable to get a valid response from the recipient's email server.
	Unknown,
}

/// Details about the email verification used for debugging.
#[derive(Debug, Deserialize, Serialize)]
pub struct DebugDetails {
	/// Identifier for the service currently running Reacher.
	pub backend_name: String,
	/// The time when the email verification started.
	pub start_time: DateTime<Utc>,
	/// The time when the email verification ended.
	pub end_time: DateTime<Utc>,
	/// The duration of the email verification.
	pub duration: Duration,
	/// Details about the email verification used for debugging.
	pub smtp: SmtpDebug,
}

impl Default for DebugDetails {
	fn default() -> Self {
		Self {
			start_time: SystemTime::now().into(),
			end_time: SystemTime::now().into(),
			duration: Duration::default(),
			smtp: SmtpDebug::default(),
			backend_name: "backend-dev".into(),
		}
	}
}

/// The result of the [check_email](check_email) function.
#[derive(Debug)]
pub struct CheckEmailOutput {
	/// Input by the user.
	pub input: String,
	pub is_reachable: Reachable,
	/// Misc details about the email address.
	pub misc: Result<MiscDetails, MiscError>,
	/// Details about the MX host.
	pub mx: Result<MxDetails, MxError>,
	/// Details about the SMTP responses of the email.
	pub smtp: Result<SmtpDetails, SmtpError>,
	/// Details about the email address.
	pub syntax: SyntaxDetails,
	/// Details about the email verification used for debugging.
	pub debug: DebugDetails,
}

impl Default for CheckEmailOutput {
	fn default() -> Self {
		CheckEmailOutput {
			input: String::default(),
			is_reachable: Reachable::Unknown,
			misc: Ok(MiscDetails::default()),
			mx: Ok(MxDetails::default()),
			smtp: Ok(SmtpDetails::default()),
			syntax: SyntaxDetails::default(),
			debug: DebugDetails::default(),
		}
	}
}

// Implement a custom serialize.
impl Serialize for CheckEmailOutput {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// This is just used internally to get the nested error field.
		#[derive(Serialize)]
		struct MyError<E> {
			error: E,
			// We add an optional "description" field when relevant, given by
			// the `get_description` on SmtpError.
			#[serde(skip_serializing_if = "Option::is_none")]
			description: Option<SmtpErrorDesc>,
		}

		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry("input", &self.input)?;
		map.serialize_entry("is_reachable", &self.is_reachable)?;
		match &self.misc {
			Ok(t) => map.serialize_entry("misc", &t)?,
			Err(error) => map.serialize_entry(
				"misc",
				&MyError {
					error,
					description: None,
				},
			)?,
		}
		match &self.mx {
			Ok(t) => map.serialize_entry("mx", &t)?,
			Err(error) => map.serialize_entry(
				"mx",
				&MyError {
					error,
					description: None,
				},
			)?,
		}
		match &self.smtp {
			Ok(t) => map.serialize_entry("smtp", &t)?,
			Err(error) => map.serialize_entry(
				"smtp",
				&MyError {
					error,
					description: error.get_description(),
				},
			)?,
		}
		map.serialize_entry("syntax", &self.syntax)?;
		map.serialize_entry("debug", &self.debug)?;
		map.end()
	}
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize, Default)]
pub struct WebdriverConfig {
	pub binary: Option<String>,
}

#[cfg(test)]
mod tests {
	use super::{CheckEmailOutput, DebugDetails};
	use async_smtp::response::{Category, Code, Detail, Response, Severity};

	#[test]
	fn should_serialize_correctly() {
		// create a dummy CheckEmailOutput, with a given message as a transient
		// SMTP error.
		fn dummy_response_with_message(m: &str) -> CheckEmailOutput {
			let r = Response::new(
				Code {
					severity: Severity::TransientNegativeCompletion,
					category: Category::MailSystem,
					detail: Detail::Zero,
				},
				vec![m.to_string(), "8BITMIME".to_string(), "SIZE 42".to_string()],
			);

			CheckEmailOutput {
				input: "foo".to_string(),
				is_reachable: super::Reachable::Unknown,
				misc: Ok(super::MiscDetails::default()),
				mx: Ok(super::MxDetails::default()),
				syntax: super::SyntaxDetails::default(),
				smtp: Err(super::SmtpError::AsyncSmtpError(r.into())),
				debug: DebugDetails::default(),
			}
		}

		let res = dummy_response_with_message("blacklist");
		let actual = serde_json::to_string(&res).unwrap();
		// Make sure the `description` is present with IpBlacklisted.
		let expected = r#""smtp":{"error":{"type":"AsyncSmtpError","message":"transient: blacklist; 8BITMIME; SIZE 42"},"description":"IpBlacklisted"}"#;
		assert!(actual.contains(expected));

		let res =
			dummy_response_with_message("Client host rejected: cannot find your reverse hostname");
		let actual = serde_json::to_string(&res).unwrap();
		// Make sure the `description` is present with NeedsRDNs.
		let expected = r#"smtp":{"error":{"type":"AsyncSmtpError","message":"transient: Client host rejected: cannot find your reverse hostname; 8BITMIME; SIZE 42"},"description":"NeedsRDNS"}"#;
		assert!(actual.contains(expected));

		let res = dummy_response_with_message("foobar");
		let actual = serde_json::to_string(&res).unwrap();
		// Make sure the `description` is NOT present.
		let expected = r#""smtp":{"error":{"type":"AsyncSmtpError","message":"transient: foobar; 8BITMIME; SIZE 42"}}"#;
		assert!(actual.contains(expected));
	}
}
