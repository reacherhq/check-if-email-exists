// Reacher - Email Verification
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

//! Helper functions to send events to Sentry.
//!
//! This module also contains functions that check if the error's given by
//! `check-if-email-exists` are known errors, in which case we don't log them
//! to Sentry.

use super::sentry_util;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{smtp::SmtpError, CheckEmailOutput};
use sentry::protocol::{Event, Level, Value};
use std::io::Error as IoError;
use std::{collections::BTreeMap, env};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Setup Sentry.
pub fn setup_sentry() -> sentry::ClientInitGuard {
	// Use an empty string if we don't have any env variable for sentry. Sentry
	// will just silently ignore.
	let sentry = sentry::init(env::var("RCH_SENTRY_DSN").unwrap_or_else(|_| "".into()));
	if sentry.is_enabled() {
		log::info!(target: "reacher", "Sentry is successfully set up.")
	}

	sentry
}

/// If BACKEND_NAME environment variable is set, add it to the sentry `extra`
/// properties.
/// For backwards compatibility, we also support HEROKU_APP_NAME env variable.
fn add_backend_name(mut extra: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
	if let Ok(n) = env::var("BACKEND_NAME") {
		extra.insert("BACKEND_NAME".into(), n.into());
	} else if let Ok(n) = env::var("HEROKU_APP_NAME") {
		extra.insert("BACKEND_NAME".into(), n.into());
	}

	extra
}

/// Helper function to send an Info event to Sentry. We use these events for
/// analytics purposes (I know, Sentry shouldn't be used for that...).
/// TODO https://github.com/reacherhq/backend/issues/207
pub fn metrics(message: String, duration: u128, domain: &str) {
	log::info!(target: "reacher", "Sending info to Sentry: {}", message);

	let mut extra = BTreeMap::new();

	extra.insert("duration".into(), duration.to_string().into());
	extra.insert("domain".into(), domain.into());
	extra = add_backend_name(extra);

	sentry::capture_event(Event {
		extra,
		level: Level::Info,
		message: Some(message),
		release: Some(CARGO_PKG_VERSION.into()),
		..Default::default()
	});
}

/// Helper function to send an Error event to Sentry. We redact all sensitive
/// info before sending to Sentry, but removing all instances of `username`.
pub fn error(message: String, result: Option<&str>, username: &str) {
	let redacted_message = redact(message.as_str(), username);
	log::debug!(target: "reacher", "Sending error to Sentry: {}", redacted_message);

	let mut extra = BTreeMap::new();
	if let Some(result) = result {
		extra.insert("CheckEmailOutput".into(), redact(result, username).into());
	}

	extra = add_backend_name(extra);

	sentry::capture_event(Event {
		extra,
		level: Level::Error,
		message: Some(redacted_message),
		release: Some(CARGO_PKG_VERSION.into()),
		..Default::default()
	});
}

/// Function to replace all usernames from email, and replace them with
/// `***@domain.com` for privacy reasons.
fn redact(input: &str, username: &str) -> String {
	input.replace(username, "***")
}

/// Check if the message contains known SMTP IO errors.
fn has_smtp_io_errors(error: &IoError) -> bool {
	// code: 104, kind: ConnectionReset, message: "Connection reset by peer",
	error.raw_os_error() == Some(104) ||
	// kind: Other, error: "incomplete",
	error.to_string() == "incomplete"
}

/// Check if the message contains known SMTP Transient errors.
fn has_smtp_transient_errors(message: &[String]) -> bool {
	let first_line = message[0].to_lowercase();

	// 4.3.2 Please try again later
	first_line.contains("try again") ||
	// Temporary local problem - please try later
	first_line.contains("try later")
}

/// Checks if the output from `check-if-email-exists` has a known error, in
/// which case we don't log to Sentry to avoid spamming it.
pub fn log_unknown_errors(result: &CheckEmailOutput) {
	match (&result.misc, &result.mx, &result.smtp) {
		(Err(error), _, _) => {
			// We log misc errors.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				result.syntax.username.as_str(),
			);
		}
		(_, Err(error), _) => {
			// We log mx errors.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				result.syntax.username.as_str(),
			);
		}
		(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
			if has_smtp_transient_errors(&response.message) =>
		{
			log::debug!(target: "reacher", "Transient error: {}", response.message[0]);
		}
		(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Io(err)))) if has_smtp_io_errors(err) => {
			log::debug!(target: "reacher", "Io error: {}", err);
		}
		(_, _, Err(error)) => {
			// If it's a SMTP error we didn't catch above, we log to
			// Sentry, to be able to debug them better. We don't want to
			// spam Sentry and log all instances of the error, hence the
			// `count` check.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				result.syntax.username.as_str(),
			);
		}
		// If everything is ok, we just return the result.
		(Ok(_), Ok(_), Ok(_)) => {}
	}
}

#[cfg(test)]
mod tests {
	use super::redact;

	#[test]
	fn test_redact() {
		assert_eq!("***@gmail.com", redact("someone@gmail.com", "someone"));
		assert_eq!(
			"my email is ***@gmail.com.",
			redact("my email is someone@gmail.com.", "someone")
		);
		assert_eq!(
			"my email is ***@gmail.com., I repeat, my email is ***@gmail.com.",
			redact(
				"my email is someone@gmail.com., I repeat, my email is someone@gmail.com.",
				"someone"
			)
		);
		assert_eq!(
			"*** @ gmail . com",
			redact("someone @ gmail . com", "someone")
		);
		assert_eq!("*** is here.", redact("someone is here.", "someone"));
	}
}
