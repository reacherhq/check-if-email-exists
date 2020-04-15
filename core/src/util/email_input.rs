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

/// Perform the email verification via a specified proxy. The usage of a proxy
/// is optional.
pub struct EmailInputProxy {
	/// Use the specified SOCKS5 proxy host to perform email verification.
	pub host: String,
	/// Use the specified SOCKS5 proxy port to perform email verification.
	pub port: u16,
}

/// Builder pattern for the input argument into the main `email_exists`
/// function.
pub struct EmailInput {
	/// The email to validate.
	pub to_email: String,
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
	pub proxy: Option<EmailInputProxy>,
}

impl EmailInput {
	/// Create a new EmailInput.
	pub fn new(email: String) -> EmailInput {
		EmailInput {
			to_email: email,
			from_email: "user@example.org".into(),
			hello_name: "localhost".into(),
			proxy: None,
		}
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command.
	pub fn from_email(&mut self, email: String) -> &mut EmailInput {
		self.from_email = email;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command.
	pub fn hello_name(&mut self, name: String) -> &mut EmailInput {
		self.hello_name = name;
		self
	}

	/// Use the specified proxy to perform email verification.
	pub fn proxy(&mut self, proxy_host: String, proxy_port: u16) -> &mut EmailInput {
		self.proxy = Some(EmailInputProxy {
			host: proxy_host,
			port: proxy_port,
		});
		self
	}
}
