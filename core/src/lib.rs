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
//! use check_if_email_exists::{check_emails, CheckEmailInput};
//!
//! async fn check() {
//!     // Let's say we want to test the deliverability of someone@gmail.com.
//!     let mut input = CheckEmailInput::new(vec!["someone@gmail.com".into()]);
//!
//!     // Optionally, we can also tweak the configuration parameters used in the
//!     // verification.
//!     input
//!         .from_email("me@example.org".into()) // Used in the `MAIL FROM:` command
//!         .hello_name("example.org".into()); // Used in the `EHLO` command
//!
//!     // Verify this input, using async/await syntax.
//!     let result = check_emails(&input).await;
//!
//!     // `result` is a `Vec<CheckEmailOutput>`, where the CheckEmailOutput
//!     // struct contains all information about one email.
//!     println!("{:?}", result);
//! }
//! ```

#[macro_use]
extern crate log;

mod misc;
mod mx;
mod smtp;
mod syntax;
mod util;

use async_smtp::{smtp::SMTP_PORT, EmailAddress};
use futures::future;
use misc::check_misc;
use mx::check_mx;
use smtp::check_smtp;
use std::str::FromStr;
use syntax::check_syntax;
use util::constants::LOG_TARGET;

pub use util::input_output::*;

/// Check a single emails. This assumes this `input.check_emails` contains
/// exactly one element. If it contains more, elements other than the first
/// one will be ignored.
///
/// # Panics
///
/// This function panics if `input.check_emails` is empty.
async fn check_single_email(input: CheckEmailInput) -> CheckEmailOutput {
	let from_email = EmailAddress::from_str(input.from_email.as_ref()).unwrap_or_else(|_| {
		warn!(
			"Inputted from_email \"{}\" is not a valid email, using \"user@example.org\" instead",
			input.from_email
		);
		EmailAddress::from_str("user@example.org").expect("This is a valid email. qed.")
	});

	let to_email = &input.to_emails[0];

	debug!(target: LOG_TARGET, "Checking email \"{}\"", to_email);
	let my_syntax = check_syntax(to_email.as_ref());
	if !my_syntax.is_valid_syntax {
		return CheckEmailOutput {
			input: to_email.to_string(),
			syntax: my_syntax,
			..Default::default()
		};
	}

	debug!(
		target: LOG_TARGET,
		"Found the following syntax validation: {:?}", my_syntax
	);

	let my_mx = match check_mx(&my_syntax).await {
		Ok(m) => m,
		e => {
			return CheckEmailOutput {
				input: to_email.to_string(),
				mx: e,
				syntax: my_syntax,
				..Default::default()
			}
		}
	};
	debug!(
		target: LOG_TARGET,
		"Found the following MX hosts {:?}", my_mx
	);

	// Return if we didn't find any MX records.
	if my_mx.lookup.is_err() {
		return CheckEmailOutput {
			input: to_email.to_string(),
			mx: Ok(my_mx),
			syntax: my_syntax,
			..Default::default()
		};
	}

	let my_misc = check_misc(
		&my_syntax
			.address
			.as_ref()
			.expect("We already checked that the email has valid format. qed.")
			.as_ref(),
	);
	debug!(
		target: LOG_TARGET,
		"Found the following misc details: {:?}", my_misc
	);

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

	CheckEmailOutput {
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
pub async fn check_emails(inputs: &CheckEmailInput) -> Vec<CheckEmailOutput> {
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
