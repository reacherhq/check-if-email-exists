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

/// Builder pattern for the input argument into the main `email_exists`
/// function.
pub struct EmailInput {
	/// The email to validate.
	pub to_email: String,
	/// Email to use in the `MAIL FROM:` SMTP command.
	/// Defaults to "user@example.org".
	pub from_email: String,
	/// Name to use in the `EHLO:` SMTP command.
	/// Defaults to "localhost" (note: "localhost" is not a FQDN).
	pub hello_name: String,
}

impl EmailInput {
	/// Create a new EmailInput.
	pub fn new(email: String) -> EmailInput {
		EmailInput {
			to_email: email,
			from_email: "user@example.org".into(),
			hello_name: "localhost".into(),
		}
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command.
	pub fn from_email<'a>(&'a mut self, email: String) -> &'a mut EmailInput {
		self.from_email = email;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command.
	pub fn hello_name<'a>(&'a mut self, name: String) -> &'a mut EmailInput {
		self.hello_name = name;
		self
	}
}
