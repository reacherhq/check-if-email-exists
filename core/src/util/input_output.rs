// check-if-email-exists
// Copyright (C) 2018-2021 Reacher

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
use crate::smtp::{SmtpDetails, SmtpError};
use crate::syntax::SyntaxDetails;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::time::Duration;

/// Perform the email verification via a specified proxy. The usage of a proxy
/// is optional.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckEmailInputProxy {
	/// Use the specified SOCKS5 proxy host to perform email verification.
	pub host: String,
	/// Use the specified SOCKS5 proxy port to perform email verification.
	pub port: u16,
}

/// Builder pattern for the input argument into the main `email_exists`
/// function.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckEmailInput {
	/// The email to validate.
	pub to_emails: Vec<String>,
	/// Email to use in the `MAIL FROM:` SMTP command.
	///
	/// Defaults to "user@example.org".
	pub from_email: String,
	/// Name to use in the `EHLO:` SMTP command.
	///
	/// Defaults to "localhost" (note: "localhost" is not a FQDN).
	pub hello_name: String,
	/// Perform the email verification via the specified SOCK5 proxy. The usage of a
	/// proxy is optional.
	pub proxy: Option<CheckEmailInputProxy>,
	/// Add optional timeout for the SMTP verification step.
	pub smtp_timeout: Option<Duration>,
	/// For Yahoo email addresses, use Yahoo's API instead of connecting
	/// directly to their SMTP servers.
	///
	/// Defaults to true.
	pub yahoo_use_api: bool,
}

impl Default for CheckEmailInput {
	fn default() -> Self {
		CheckEmailInput {
			to_emails: vec![],
			from_email: "user@example.org".into(),
			hello_name: "localhost".into(),
			proxy: None,
			smtp_timeout: None,
			yahoo_use_api: true,
		}
	}
}

impl CheckEmailInput {
	/// Create a new CheckEmailInput.
	pub fn new(to_emails: Vec<String>) -> CheckEmailInput {
		CheckEmailInput {
			to_emails,
			..Default::default()
		}
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command. Defaults to
	/// `user@example.org` if not explicitly set.
	#[deprecated(since = "0.8.24", note = "Please with with_from_email instead")]
	pub fn from_email(&mut self, email: String) -> &mut CheckEmailInput {
		self.from_email = email;
		self
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command. Defaults to
	/// `user@example.org` if not explicitly set.
	pub fn with_from_email(&mut self, email: String) -> &mut CheckEmailInput {
		self.from_email = email;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command. Defaults to `localhost`
	/// if not explicitly set.
	#[deprecated(since = "0.8.24", note = "Please with with_hello_name instead")]
	pub fn hello_name(&mut self, name: String) -> &mut CheckEmailInput {
		self.hello_name = name;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command. Defaults to `localhost`
	/// if not explicitly set.
	pub fn with_hello_name(&mut self, name: String) -> &mut CheckEmailInput {
		self.hello_name = name;
		self
	}

	/// Use the specified SOCK5 proxy to perform email verification.
	#[deprecated(since = "0.8.24", note = "Please with with_proxy instead")]
	pub fn proxy(&mut self, proxy_host: String, proxy_port: u16) -> &mut CheckEmailInput {
		self.proxy = Some(CheckEmailInputProxy {
			host: proxy_host,
			port: proxy_port,
		});
		self
	}

	/// Use the specified SOCK5 proxy to perform email verification.
	pub fn with_proxy(&mut self, proxy: CheckEmailInputProxy) -> &mut CheckEmailInput {
		self.proxy = Some(proxy);
		self
	}

	/// Add optional timeout for the SMTP verification step.
	#[deprecated(since = "0.8.24", note = "Please with with_smtp_timeout instead")]
	pub fn smtp_timeout(&mut self, duration: Duration) -> &mut CheckEmailInput {
		self.smtp_timeout = Some(duration);
		self
	}

	/// Add optional timeout for the SMTP verification step.
	pub fn with_smtp_timeout(&mut self, duration: Duration) -> &mut CheckEmailInput {
		self.smtp_timeout = Some(duration);
		self
	}

	/// Set whether to use Yahoo's API or connecting directly to their SMTP
	/// servers. Defaults to true.
	#[deprecated(since = "0.8.24", note = "Please with with_yahoo_use_api instead")]
	pub fn yahoo_use_api(&mut self, use_api: bool) -> &mut CheckEmailInput {
		self.yahoo_use_api = use_api;
		self
	}

	/// Set whether to use Yahoo's API or connecting directly to their SMTP
	/// servers. Defaults to true.
	pub fn with_yahoo_use_api(&mut self, use_api: bool) -> &mut CheckEmailInput {
		self.yahoo_use_api = use_api;
		self
	}
}

/// An enum to describe how confident we are that the recipient address is
/// real.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
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
		}

		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry("input", &self.input)?;
		map.serialize_entry("is_reachable", &self.is_reachable)?;
		match &self.misc {
			Ok(t) => map.serialize_entry("misc", &t)?,
			Err(error) => map.serialize_entry("misc", &MyError { error })?,
		}
		match &self.mx {
			Ok(t) => map.serialize_entry("mx", &t)?,
			Err(error) => map.serialize_entry("mx", &MyError { error })?,
		}
		match &self.smtp {
			Ok(t) => map.serialize_entry("smtp", &t)?,
			Err(error) => map.serialize_entry("smtp", &MyError { error })?,
		}
		map.serialize_entry("syntax", &self.syntax)?;
		map.end()
	}
}
