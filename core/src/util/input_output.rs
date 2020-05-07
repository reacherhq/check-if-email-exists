// check-if-email-exists
// Copyright (C) 2018-2020 Amaury Martiny

// check-if-email-exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check-if-email-exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check-if-email-exists.  If not, see <http://www.gnu.org/licenses/>.

use crate::misc::{MiscDetails, MiscError};
use crate::mx::{MxDetails, MxError};
use crate::smtp::{SmtpDetails, SmtpError};
use crate::syntax::SyntaxDetails;
use serde::{ser::SerializeMap, Serialize, Serializer};

/// Perform the email verification via a specified proxy. The usage of a proxy
/// is optional.
#[derive(Debug, Clone)]
pub struct CheckEmailInputProxy {
	/// Use the specified SOCKS5 proxy host to perform email verification.
	pub host: String,
	/// Use the specified SOCKS5 proxy port to perform email verification.
	pub port: u16,
}

/// Builder pattern for the input argument into the main `email_exists`
/// function.
#[derive(Debug, Clone)]
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
	/// Perform the email verification via a specified proxy. The usage of a
	/// proxy is optional.
	pub proxy: Option<CheckEmailInputProxy>,
}

impl CheckEmailInput {
	/// Create a new CheckEmailInput.
	pub fn new(to_emails: Vec<String>) -> CheckEmailInput {
		CheckEmailInput {
			to_emails,
			from_email: "user@example.org".into(),
			hello_name: "localhost".into(),
			proxy: None,
		}
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command.
	pub fn from_email(&mut self, email: String) -> &mut CheckEmailInput {
		self.from_email = email;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command.
	pub fn hello_name(&mut self, name: String) -> &mut CheckEmailInput {
		self.hello_name = name;
		self
	}

	/// Use the specified proxy to perform email verification.
	pub fn proxy(&mut self, proxy_host: String, proxy_port: u16) -> &mut CheckEmailInput {
		self.proxy = Some(CheckEmailInputProxy {
			host: proxy_host,
			port: proxy_port,
		});
		self
	}
}

/// The result of the [check_email](check_email) function.
#[derive(Debug)]
pub struct CheckEmailOutput {
	/// Input by the user.
	pub input: String,
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
