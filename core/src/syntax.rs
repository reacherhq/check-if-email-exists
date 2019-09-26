// check_if_email_exists
// Copyright (C) 2018-2019 Amaury Martiny

// check_if_email_exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check_if_email_exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check_if_email_exists.  If not, see <http://www.gnu.org/licenses/>.

use lettre::error::Error as LettreError;
/// Information about the syntax of an email address
use lettre::EmailAddress;
use serde::{Serialize, Serializer};
use std::str::FromStr;

/// Syntax information after parsing an email address
#[derive(Debug, PartialEq, Serialize)]
pub struct SyntaxDetails {
	/// The email address as a lettre EmailAddress
	pub address: EmailAddress,
	/// The domain name, after "@"
	pub domain: String,
	/// The username, before "@"
	pub username: String,
	/// Is the email in a valid format?
	pub valid_format: bool,
}

#[derive(Debug)]
pub struct SyntaxError(LettreError);

impl PartialEq for SyntaxError {
	fn eq(&self, other: &Self) -> bool {
		match (self.0, other.0) {
			(LettreError::InvalidEmailAddress, LettreError::InvalidEmailAddress) => true,
			_ => false,
		}
	}
}

impl Serialize for SyntaxError {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.collect_str(&self.0)
	}
}

/// From an `email_address` string, compute syntax information about it, such as
/// username and domain
pub fn address_syntax(email_address: &str) -> Result<SyntaxDetails, SyntaxError> {
	let email_address = match EmailAddress::from_str(email_address) {
		Ok(m) => m,
		Err(error) => return Err(SyntaxError(error)),
	};

	let iter: &str = email_address.as_ref();
	let mut iter = iter.split("@");
	let username = iter
		.next()
		.expect("We checked above that email is valid. qed.")
		.into();
	let domain = iter
		.next()
		.expect("We checked above that email is valid. qed.")
		.into();

	let address_details = SyntaxDetails {
		address: email_address,
		domain,
		username,
		valid_format: true,
	};

	Ok(address_details)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_return_error_for_invalid_email() {
		assert_eq!(
			address_syntax("foo"),
			Err(SyntaxError(LettreError::InvalidEmailAddress))
		);
	}

	#[test]
	fn should_return_error_for_invalid_email_with_at() {
		assert_eq!(
			address_syntax("foo@bar"),
			Err(SyntaxError(LettreError::InvalidEmailAddress))
		);
	}

	#[test]
	fn should_work_for_valid_email() {
		assert_eq!(
			address_syntax("foo@bar.com"),
			Ok(SyntaxDetails {
				address: EmailAddress::new("foo@bar.com".into()).unwrap(),
				domain: "bar.com".into(),
				username: "foo".into(),
				valid_format: true
			})
		);
	}
}
