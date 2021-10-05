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
//! use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailInputProxy};
//!
//! async fn check() {
//!     // Let's say we want to test the deliverability of someone@gmail.com.
//!     let mut input = CheckEmailInput::new(vec!["someone@gmail.com".into()]);
//!
//!     // Optionally, we can also tweak the configuration parameters used in the
//!     // verification.
//!     input
//!         .set_from_email("me@example.org".into()) // Used in the `MAIL FROM:` command
//!         .set_hello_name("example.org".into())    // Used in the `EHLO` command
//!         .set_smtp(587)                           // Use port 587 instead of 25
//!         .set_proxy(CheckEmailInputProxy {        // Use a SOCKS5 proxy to verify the email
//!             host: "my-proxy.io".into(),
//!             port: 1080
//!     });
//!
//!     // Verify this input, using async/await syntax.
//!     let result = check_email(&input).await;
//!
//!     // `result` is a `Vec<CheckEmailOutput>`, where the CheckEmailOutput
//!     // struct contains all information about one email.
//!     println!("{:?}", result);
//! }
//! ```

pub mod misc;
pub mod mx;
pub mod smtp;
pub mod syntax;
mod util;

use futures::future;
use misc::{check_misc, MiscDetails};
use mx::check_mx;
use smtp::{check_smtp, SmtpDetails, SmtpError};
use syntax::check_syntax;
use util::constants::LOG_TARGET;

pub use util::input_output::*;

/// Given an email's misc and smtp details, calculate an estimate of our
/// confidence on how reachable the email is.
fn calculate_reachable(misc: &MiscDetails, smtp: &Result<SmtpDetails, SmtpError>) -> Reachable {
	if let Ok(smtp) = smtp {
		if misc.is_disposable || misc.is_role_account || smtp.is_catch_all || smtp.has_full_inbox {
			return Reachable::Risky;
		}

		if !smtp.is_deliverable || !smtp.can_connect_smtp || smtp.is_disabled {
			return Reachable::Invalid;
		}

		Reachable::Safe
	} else {
		Reachable::Unknown
	}
}

/// Check a single emails. This assumes this `input.check_email` contains
/// exactly one element. If it contains more, elements other than the first
/// one will be ignored.
///
/// # Panics
///
/// This function panics if `input.check_email` is empty.
async fn check_single_email(input: CheckEmailInput) -> CheckEmailOutput {
	let to_email = &input.to_emails[0];

	log::debug!(target: LOG_TARGET, "Checking email \"{}\"", to_email);
	let my_syntax = check_syntax(to_email.as_ref());
	if !my_syntax.is_valid_syntax {
		return CheckEmailOutput {
			input: to_email.to_string(),
			is_reachable: Reachable::Invalid,
			syntax: my_syntax,
			..Default::default()
		};
	}

	log::debug!(
		target: LOG_TARGET,
		"Found the following syntax validation: {:?}",
		my_syntax
	);

	let my_mx = match check_mx(&my_syntax).await {
		Ok(m) => m,
		e => {
			// This happens when there's an internal error while checking MX
			// records. Should happen fairly rarely.
			return CheckEmailOutput {
				input: to_email.to_string(),
				is_reachable: Reachable::Unknown,
				mx: e,
				syntax: my_syntax,
				..Default::default()
			};
		}
	};
	log::debug!(
		target: LOG_TARGET,
		"Found the following MX hosts {:?}",
		my_mx
	);

	// Return if we didn't find any MX records.
	if my_mx.lookup.is_err() {
		return CheckEmailOutput {
			input: to_email.to_string(),
			is_reachable: Reachable::Invalid,
			mx: Ok(my_mx),
			syntax: my_syntax,
			..Default::default()
		};
	}

	let my_misc = check_misc(&my_syntax);
	log::debug!(
		target: LOG_TARGET,
		"Found the following misc details: {:?}",
		my_misc
	);

	// We simply take the first lookup result, and connect to that SMTP server.
	// FIXME Add retry mechanism to other lookup results.
	let my_smtp = my_mx
		.lookup
		.as_ref()
		.expect("If lookup is error, we already returned. qed.")
		.iter()
		.next()
		.map(|host| {
			check_smtp(
				my_syntax
					.address
					.as_ref()
					.expect("We already checked that the email has valid format. qed."),
				host.exchange(),
				input.smtp_port,
				my_syntax.domain.as_ref(),
				&input,
			)
		})
		.expect("Lookup cannot be empty. qed.")
		.await;

	CheckEmailOutput {
		input: to_email.to_string(),
		is_reachable: calculate_reachable(&my_misc, &my_smtp),
		misc: Ok(my_misc),
		mx: Ok(my_mx),
		smtp: my_smtp,
		syntax: my_syntax,
	}
}

/// The main function of this library: takes as input a list of email addresses
/// to check. Then performs syntax, mx, smtp and misc checks, and outputs a
/// list of results.
///
/// Please note that checking multiple emails at once (by putting multiple
/// emails in the `inputs.to_emails` Vec) is still a beta feature, and not
/// fully optimized. For more info, see #65
/// <https://github.com/reacherhq/check-if-email-exists/issues/65>.
pub async fn check_email(inputs: &CheckEmailInput) -> Vec<CheckEmailOutput> {
	// FIXME Obviously, the below `join_all` is not optimal. Some optimizations
	// include:
	// - if multiple email addresses share the same domain, we should only do
	// `check_mx` call for all these email addresses.
	// - if multiple email addresses share the same domain, we should call
	// `check_smtp` with grouped email addresses, to share a SMTP connection.
	// ref: https://github.com/reacherhq/check-if-email-exists/issues/65.
	let inputs = inputs.to_emails.iter().map(|email| {
		// Create n `CheckEmailInput`s, each with one email address.
		CheckEmailInput {
			to_emails: vec![email.clone()],
			from_email: inputs.from_email.clone(),
			hello_name: inputs.hello_name.clone(),
			proxy: inputs.proxy.clone(),
			smtp_port: inputs.smtp_port,
			smtp_timeout: inputs.smtp_timeout,
			yahoo_use_api: inputs.yahoo_use_api,
		}
	});
	future::join_all(inputs.map(check_single_email)).await
}
