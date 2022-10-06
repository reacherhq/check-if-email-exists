// check-if-email-exists
// Copyright (C) 2018-2022 Reacher

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

use async_smtp::EmailAddress;
use levenshtein::levenshtein;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Syntax information after parsing an email address
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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
	pub suggestion: Option<String>,
}

impl Default for SyntaxDetails {
	fn default() -> Self {
		SyntaxDetails {
			address: None,
			domain: "".into(),
			is_valid_syntax: false,
			username: "".into(),
			suggestion: None,
		}
	}
}

/// From an `email_address` string, compute syntax information about it, such as
/// username and domain.
pub fn check_syntax(email_address: &str) -> SyntaxDetails {
	let email_address = match EmailAddress::from_str(email_address) {
		Ok(m) => {
			if mailchecker::is_valid(email_address) {
				m
			} else {
				return SyntaxDetails {
					address: None,
					domain: "".into(),
					is_valid_syntax: false,
					username: "".into(),
					suggestion: None,
				};
			}
		}
		_ => {
			return SyntaxDetails {
				address: None,
				domain: "".into(),
				is_valid_syntax: false,
				username: "".into(),
				suggestion: None,
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
		suggestion: None,
	}
}

const MAIL_PROVIDERS: &[&str] = &[
	"gmail.com",
	"yahoo.com",
	"outlook.com",
	"hotmail.com",
	"protonmail.com",
	"icloud.com",
	"yandex.com",
];
// Supplies the syntax parameter with a suggestion that matches the mail domain best by levenshtein
// distance.
pub fn get_similar_mail_provider(syntax: &mut SyntaxDetails) {
	for possible_provider in MAIL_PROVIDERS {
		let distance = levenshtein(&syntax.domain, possible_provider);

		if distance < 3 {
			// Return full address
			syntax.suggestion = Some(format!(
				"{}@{}",
				syntax.username,
				String::from(*possible_provider),
			));
			break;
		}
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
				suggestion: None,
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
				suggestion: None,
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
				suggestion: None,
			}
		);
	}

	#[test]
	fn should_suggest_a_correct_mail_if_similar() {
		let mut syntax = SyntaxDetails {
			address: Some(EmailAddress::new("test@gmali.com".into()).unwrap()),
			domain: "gmali.com".into(),
			is_valid_syntax: true,
			username: "test".into(),
			suggestion: None,
		};
		get_similar_mail_provider(&mut syntax);
		assert_eq!(syntax.suggestion, Some("test@gmail.com".to_string()))
	}
}
