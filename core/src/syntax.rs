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

use async_smtp::EmailAddress;
use serde::Serialize;
use std::str::FromStr;

/// Syntax information after parsing an email address
#[derive(Debug, PartialEq, Serialize)]
pub struct SyntaxDetails {
	/// The email address as a async_smtp `EmailAddress`. It will be `None` if
	/// the email address is ill-formed.
	pub address: Option<EmailAddress>,
	/// The domain name, after "@". It will be the empty string if the email
	/// address if ill-formed.
	pub domain: String,
	/// Does the email have a valid syntax?
	pub is_valid_syntax: bool,
	/// The username, before "@". It will be the empty string if the email
	/// address if ill-formed.
	pub username: String,
}

/// From an `email_address` string, compute syntax information about it, such as
/// username and domain.
pub fn check_syntax(email_address: &str) -> SyntaxDetails {
	let email_address = match EmailAddress::from_str(email_address) {
		Ok(m) => m,
		_ => {
			return SyntaxDetails {
				address: None,
				domain: "".into(),
				is_valid_syntax: false,
				username: "".into(),
			}
		}
	};

	let iter: &str = email_address.as_ref();
	let mut iter = iter.split('@');
	let username = iter
		.next()
		.expect("We checked above that email is valid. qed.")
		.into();
	let domain = iter
		.next()
		.expect("We checked above that email is valid. qed.")
		.into();

	SyntaxDetails {
		address: Some(email_address),
		domain,
		is_valid_syntax: true,
		username,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_return_invalid_for_invalid_email() {
		assert_eq!(
			check_syntax("foo"),
			SyntaxDetails {
				address: None,
				domain: "".into(),
				is_valid_syntax: false,
				username: "".into(),
			}
		);
	}

	#[test]
	fn should_return_invalid_for_invalid_email_with_at() {
		assert_eq!(
			check_syntax("foo@bar"),
			SyntaxDetails {
				address: None,
				domain: "".into(),
				is_valid_syntax: false,
				username: "".into(),
			}
		);
	}

	#[test]
	fn should_work_for_valid_email() {
		assert_eq!(
			check_syntax("foo@bar.com"),
			SyntaxDetails {
				address: Some(EmailAddress::new("foo@bar.com".into()).unwrap()),
				domain: "bar.com".into(),
				is_valid_syntax: true,
				username: "foo".into(),
			}
		);
	}
}
