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
//!     let mut input = CheckEmailInput::new("someone@gmail.com".into());
//!
//!     // Optionally, we can also tweak the configuration parameters used in the
//!     // verification.
//!     input
//!         .set_from_email("me@example.org".into()) // Used in the `MAIL FROM:` command
//!         .set_hello_name("example.org".into())    // Used in the `EHLO` command
//!         .set_smtp_port(587)                      // Use port 587 instead of 25
//!         .set_proxy(CheckEmailInputProxy {        // Use a SOCKS5 proxy to verify the email
//!             host: "my-proxy.io".into(),
//!             port: 1080,
//!             username: None,                      // You can also set it non-empty
//!             password: None
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

use misc::{check_misc, MiscDetails};
use mx::check_mx;
use smtp::{check_smtp, SmtpDetails, SmtpError};
use syntax::check_syntax;
pub use util::constants::LOG_TARGET;
pub use util::input_output::*;

/// Given an email's misc and smtp details, calculate an estimate of our
/// confidence on how reachable the email is.
///
/// Maybe we can switch to a points-based system?
/// ref: https://github.com/reacherhq/check-if-email-exists/issues/935
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

/// The main function of this library: verify a single email. Performs, in the
/// following order, 4 types of verifications:
/// - syntax check: verify the email is well-formed,
/// - MX checks: verify the domain is configured to receive email,
/// - SMTP checks: connect to the SMTP server and verify the email is
///   deliverable,
/// - misc checks: metadata about the email provider.
///
/// Returns a `CheckEmailOutput` output, whose `is_reachable` field is one of
/// `Safe`, `Invalid`, `Risky` or `Unknown`.
pub async fn check_email(input: &CheckEmailInput) -> CheckEmailOutput {
	let to_email = &input.to_email;

	log::debug!(
		target: LOG_TARGET,
		"[email={}] Checking email \"{}\"",
		to_email,
		to_email
	);
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
		"[email={}] Found the following syntax validation: {:?}",
		to_email,
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

	log::debug!(
		target: LOG_TARGET,
		"[email={}] Found the following MX hosts: {:?}",
		to_email,
		my_mx
			.lookup
			.as_ref()
			.expect("If lookup is error, we already returned. qed.")
			.iter()
			.map(|host| host.exchange().to_string())
			.collect::<Vec<String>>()
	);

	let my_misc = check_misc(&my_syntax);
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Found the following misc details: {:?}",
		to_email,
		my_misc
	);

	// We loop through all the MX records, and check each one of them. This is
	// because to prevent SPAM, some servers put a dummy server as 1st MX
	// record.
	// ref: https://github.com/reacherhq/check-if-email-exists/issues/1049
	let mut my_smtp: Option<Result<SmtpDetails, SmtpError>> = None;
	for host in my_mx
		.lookup
		.as_ref()
		.expect("If lookup is error, we already returned. qed.")
		.iter()
	{
		let res = check_smtp(
			my_syntax
				.address
				.as_ref()
				.expect("We already checked that the email has valid format. qed."),
			host.exchange(),
			input.smtp_port,
			my_syntax.domain.as_ref(),
			input,
		)
		.await;
		let is_reachable = res.is_ok();
		my_smtp = Some(res);
		if !is_reachable {
			continue; // If error, then we move on to next MX record.
		} else {
			break; // If successful, then we break.
		}
	}
	let my_smtp = my_smtp.expect(
		"As long as lookup has at least 1 element (which we checked), my_smtp will be a Some. qed.",
	);

	CheckEmailOutput {
		input: to_email.to_string(),
		is_reachable: calculate_reachable(&my_misc, &my_smtp),
		misc: Ok(my_misc),
		mx: Ok(my_mx),
		smtp: my_smtp,
		syntax: my_syntax,
	}
}
