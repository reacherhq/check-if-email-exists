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

//! `check-if-email-exists` lets you check if an email address exists without
//! sending any email.
//!
//! Under the hood, it connects to the email address's SMTP server, and,
//! analyzing the server's responses against some SMTP commands, finds out
//! information about the email address, such as:
//! - Email deliverability: Is an email sent to this address deliverable?
//! - Syntax validation. Is the address syntactically valid?
//! - DNS records validation. Does the domain of the email address have valid
//! MX DNS records?
//! - Disposable email address (DEA) validation. Is the address provided by a
//! known disposable email address provider?
//! - SMTP server validation. Can the mail exchanger of the email address
//! domain be contacted successfully?
//! - Mailbox disabled. Has this email address been disabled by the email
//! provider?
//! - Full inbox. Is the inbox of this mailbox full?
//! - Catch-all address. Is this email address a catch-all address?
//!
//! ```rust
//! use check_if_email_exists::{email_exists, CheckEmailInput};
//!
//! async fn check() {
//!     // Let's say we want to test the deliverability of someone@gmail.com.
//!     let mut input = CheckEmailInput::new("someone@gmail.com".into());
//!
//!     // Optionally, we can also tweak the configuration parameters used in the
//!     // verification.
//!     input
//!         .from_email("me@example.org".into()) // Used in the `MAIL FROM:` command
//!         .hello_name("example.org".into()); // Used in the `EHLO` command
//!
//!     // Verify this input, using async/await syntax.
//!     let result = email_exists(&input).await;
//!
//!     // `result` is a `CheckEmailResult` struct containing all information about the
//!     // email.
//!     println!("{:?}", result);
//! }
//! ```

#[macro_use]
extern crate log;

pub mod misc;
pub mod mx;
pub mod smtp;
pub mod syntax;
mod util;

use async_smtp::{smtp::SMTP_PORT, EmailAddress};
use futures::future;
use misc::{check_misc, MiscDetails, MiscError};
use mx::{check_mx, MxDetails, MxError};
use serde::{ser::SerializeMap, Serialize, Serializer};
use smtp::{check_smtp, SmtpDetails, SmtpError};
use std::str::FromStr;
use syntax::{check_syntax, SyntaxDetails};

pub use util::input::*;

/// The result of the [check_email](check_email) function.
#[derive(Debug)]
pub struct CheckEmailResult {
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

// Implement a custom serialize.
impl Serialize for CheckEmailResult {
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

/// Check a single emails. This assumes this `input.check_emails` contains
/// exactly one element. If it contains more, elements other than the first
/// one will be ignored.
///
/// # Panics
///
/// This function panics if `input.check_emails` is empty.
async fn check_single_email(input: CheckEmailInput) -> CheckEmailResult {
	let from_email = EmailAddress::from_str(input.from_email.as_ref()).unwrap_or_else(|_| {
		warn!(
			"Inputted from_email \"{}\" is not a valid email, using \"user@example.org\" instead",
			input.from_email
		);
		EmailAddress::from_str("user@example.org").expect("This is a valid email. qed.")
	});

	let to_email = &input.to_emails[0];

	debug!("Checking email \"{}\"", to_email);
	let my_syntax = check_syntax(to_email.as_ref());
	if !my_syntax.is_valid_syntax {
		return CheckEmailResult {
			input: to_email.to_string(),
			misc: Err(MiscError::Skipped),
			mx: Err(MxError::Skipped),
			smtp: Err(SmtpError::Skipped),
			syntax: my_syntax,
		};
	}

	debug!("Found the following syntax validation: {:?}", my_syntax);

	let my_mx = match check_mx(&my_syntax).await {
		Ok(m) => m,
		e => {
			return CheckEmailResult {
				input: to_email.to_string(),
				misc: Err(MiscError::Skipped),
				mx: e,
				smtp: Err(SmtpError::Skipped),
				syntax: my_syntax,
			}
		}
	};
	debug!("Found the following MX hosts {:?}", my_mx);

	// Return if we didn't find any MX records.
	if my_mx.lookup.is_err() {
		return CheckEmailResult {
			input: to_email.to_string(),
			misc: Err(MiscError::Skipped),
			mx: Ok(my_mx),
			smtp: Err(SmtpError::Skipped),
			syntax: my_syntax,
		};
	}

	let my_misc = check_misc(
		&my_syntax
			.address
			.as_ref()
			.expect("We already checked that the email has valid format. qed.")
			.as_ref(),
	);
	debug!("Found the following misc details: {:?}", my_misc);

	// Create one future per lookup result.
	let futures = my_mx
		.lookup
		.as_ref()
		.map(|lookup| {
			lookup
				.iter()
				.map(|host| {
					let fut = check_smtp(
						my_syntax
							.address
							.as_ref()
							.expect("We already checked that the email has valid format. qed."),
						&from_email,
						host.exchange(),
						// FIXME We could add ports 465 and 587 too.
						SMTP_PORT,
						my_syntax.domain.as_ref(),
						input.hello_name.as_ref(),
						&input.proxy,
					);

					// https://rust-lang.github.io/async-book/04_pinning/01_chapter.html
					Box::pin(fut)
				})
				.collect::<Vec<_>>()
		})
		.expect("If lookup is empty, we already returned. qed.");

	// Race, return the first future that resolves.
	let my_smtp = match future::select_ok(futures).await {
		Ok((details, _)) => Ok(details),
		Err(err) => Err(err),
	};

	CheckEmailResult {
		input: to_email.to_string(),
		misc: Ok(my_misc),
		mx: Ok(my_mx),
		smtp: my_smtp,
		syntax: my_syntax,
	}
}

/// The main function of this library: takes as input a list of email addresses
/// to check. Then performs syntax, mx, smtp and misc checks, and outputs a
/// list of results.
pub async fn check_emails(inputs: &CheckEmailInput) -> Vec<CheckEmailResult> {
	// FIXME Obviously, the below `join_all` is not optimal. Some optimizations
	// include:
	// - if multiple email addresses share the same domain, we should only do
	// `check_mx` call for all these email addresses.
	// - if multiple email addresses share the same domain, we should call
	// `check_smtp` with grouped email addresses, to share a SMTP connection.
	// Also see https://github.com/amaurymartiny/check-if-email-exists/issues/65.
	let inputs = inputs.to_emails.iter().map(|email| {
		// Create n `CheckEmailInput`s, each with one email address.
		CheckEmailInput {
			to_emails: vec![email.clone()],
			from_email: inputs.from_email.clone(),
			hello_name: inputs.hello_name.clone(),
			proxy: inputs.proxy.clone(),
		}
	});
	future::join_all(inputs.map(check_single_email)).await
}
