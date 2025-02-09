// check-if-email-exists
// Copyright (C) 2018-2023 Reacher

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
//!   MX DNS records?
//! - Disposable email address (DEA) validation. Is the address provided by a
//!   known disposable email address provider?
//! - SMTP server validation. Can the mail exchanger of the email address
//!   domain be contacted successfully?
//! - Mailbox disabled. Has this email address been disabled by the email
//!   provider?
//! - Full inbox. Is the inbox of this mailbox full?
//! - Catch-all address. Is this email address a catch-all address?
//!
//! ```rust
//! use check_if_email_exists::{check_email, CheckEmailInputBuilder, CheckEmailInputProxy};
//! use check_if_email_exists::smtp::verif_method::{VerifMethod, VerifMethodSmtpConfig, GmailVerifMethod};
//! use std::collections::HashMap;
//!
//! async fn check() {
//!     // Let's say we want to test the deliverability of someone@gmail.com.
//!
//!     // We can tweak how we want to verify the email address, though using
//!     // the default values is usually enough. However, if you want to use a
//!     // proxy, you can do so like this:
//!     let mut proxies = HashMap::new();
//!     proxies.insert("proxy1".to_string(), CheckEmailInputProxy {
//!         host: "my-proxy.io".to_string(),             // Use a SOCKS5 proxy to verify the email
//!         port: 1080,
//!         username: None,                              // You can also set it non-empty
//!         password: None
//!     });
//!     let verif_method = VerifMethod {
//! 		proxies,
//!         gmail: GmailVerifMethod::Smtp(VerifMethodSmtpConfig {
//!            from_email: "me@example.org".to_string(), // Used in the `MAIL FROM:` command
//!            hello_name: "example.org".to_string(),    // Used in the `EHLO` command
//!            smtp_port: 587,                           // Use port 587 instead of 25
//!            proxy: Some("proxy1".to_string()),        // Use the proxy we defined above
//! 		   ..Default::default()
//!         }),
//!         ..Default::default()
//!     };
//!
//!     let input = CheckEmailInputBuilder::default()
//!         .to_email("someone@gmail.com".into())
//!         .verif_method(verif_method)
//!         .build()
//!         .unwrap();
//!
//!     // Verify this input, using async/await syntax.
//!     let result = check_email(&input).await;
//!
//!     // `result` is a `Vec<CheckEmailOutput>`, where the CheckEmailOutput
//!     // struct contains all information about one email.
//!     println!("{:?}", result);
//! }
//! ```

mod haveibeenpwned;
pub mod misc;
pub mod mx;
mod rules;
pub mod smtp;
pub mod syntax;
mod util;

use hickory_proto::rr::rdata::MX;
use misc::{check_misc, MiscDetails};
use mx::check_mx;
use rand::Rng;
use rustls::crypto::ring;
use smtp::{check_smtp, SmtpDetails, SmtpError};
pub use smtp::{is_gmail, is_hotmail, is_hotmail_b2b, is_hotmail_b2c, is_yahoo};
use std::sync::Once;
use std::time::{Duration, SystemTime};
use syntax::{check_syntax, get_similar_mail_provider};
pub use util::input_output::*;
#[cfg(feature = "sentry")]
pub use util::sentry::*;

use crate::rules::{has_rule, Rule};

/// The target where to log check-if-email-exists logs.
pub const LOG_TARGET: &str = "reacher";

static INIT: Once = Once::new();

/// check-if-email-exists uses rustls for its TLS connections. This function
/// initializes the default crypto provider for rustls.
pub fn initialize_crypto_provider() {
	INIT.call_once(|| {
		ring::default_provider().install_default().unwrap();
	});
}

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
	initialize_crypto_provider();
	let start_time = SystemTime::now();
	let to_email = &input.to_email;

	tracing::debug!(
		target: LOG_TARGET,
		email=%to_email,
		"Checking email"
	);
	let mut my_syntax = check_syntax(to_email.as_ref());
	if !my_syntax.is_valid_syntax {
		return CheckEmailOutput {
			input: to_email.to_string(),
			is_reachable: Reachable::Invalid,
			syntax: my_syntax,
			..Default::default()
		};
	}

	tracing::debug!(
		target: LOG_TARGET,
		email=%to_email,
		syntax=?my_syntax,
		"Found syntax validation"
	);

	let my_mx = match check_mx(&my_syntax).await {
		Ok(m) => m,
		e => {
			get_similar_mail_provider(&mut my_syntax);

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
		get_similar_mail_provider(&mut my_syntax);

		return CheckEmailOutput {
			input: to_email.to_string(),
			is_reachable: Reachable::Invalid,
			mx: Ok(my_mx),
			syntax: my_syntax,
			..Default::default()
		};
	}

	let mx_hosts: Vec<String> = my_mx
		.lookup
		.as_ref()
		.expect("If lookup is error, we already returned. qed.")
		.iter()
		.map(|host| host.to_string())
		.collect();

	tracing::debug!(
		target: LOG_TARGET,
		email=%to_email,
		mx_hosts=?mx_hosts,
		"Found MX hosts"
	);

	let my_misc = check_misc(
		&my_syntax,
		input.check_gravatar,
		input.haveibeenpwned_api_key.clone(),
	)
	.await;

	tracing::debug!(
		target: LOG_TARGET,
		email=%to_email,
		misc=?my_misc,
		"Found misc details"
	);

	// From the list of MX records, we only choose one: we don't choose the
	// first or last ones, because some domains put dummy MX records at the
	// beginning or end of the list (sorted by priority). Instead, we choose a
	// random one in the middle of the list.
	//
	// See here for explanation: https://cwiki.apache.org/confluence/display/SPAMASSASSIN/OtherTricks
	//
	// If anyone has a better algorithm, let me know by creating an issue on
	// Github.
	// ref: https://github.com/reacherhq/check-if-email-exists/issues/1049
	let mut mx_records = my_mx
		.lookup
		.as_ref()
		.expect("If lookup is error, we already returned. qed.")
		.iter()
		// Don't try to connect to honey pot servers.
		.filter(|a| {
			!has_rule(
				&my_syntax.domain,
				&a.exchange().to_string(),
				&Rule::HoneyPot,
			)
		})
		.collect::<Vec<&MX>>();
	mx_records.sort_by_key(|a| a.preference());
	let host = if mx_records.len() >= 3 {
		let mut rng = rand::thread_rng();
		let index = rng.gen_range(1..mx_records.len() - 1);
		mx_records[index]
	} else {
		mx_records[mx_records.len() - 1]
	};

	let (my_smtp, smtp_debug) = check_smtp(
		my_syntax
			.address
			.as_ref()
			.expect("We already checked that the email has valid format. qed."),
		host.exchange(),
		my_syntax.domain.as_ref(),
		input,
	)
	.await;

	if my_smtp.is_err() {
		get_similar_mail_provider(&mut my_syntax);
	}

	let end_time = SystemTime::now();

	let output = CheckEmailOutput {
		input: to_email.to_string(),
		is_reachable: calculate_reachable(&my_misc, &my_smtp),
		misc: Ok(my_misc),
		mx: Ok(my_mx),
		smtp: my_smtp,
		syntax: my_syntax,
		debug: DebugDetails {
			start_time: start_time.into(),
			end_time: end_time.into(),
			duration: end_time
				.duration_since(start_time)
				.unwrap_or(Duration::from_secs(0)),
			smtp: smtp_debug,
			backend_name: input.backend_name.clone(),
		},
	};

	#[cfg(feature = "sentry")]
	log_unknown_errors(&output, &input.backend_name);

	output
}
