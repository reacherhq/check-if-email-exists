// Reacher - Email Verification
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

//! Helper functions to send events to Sentry.
//!
//! This module also contains functions that check if the error's given by
//! `check-if-email-exists` are known errors, in which case we don't log them
//! to Sentry.

use std::borrow::Cow;
use std::env;

use async_smtp::smtp::error::Error as AsyncSmtpError;
use sentry::protocol::{Event, Exception, Level, Values};
use tracing::{debug, info};

use crate::misc::MiscError;
use crate::mx::MxError;
use crate::LOG_TARGET;
use crate::{smtp::SmtpError, CheckEmailOutput};

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Setup Sentry.
pub fn setup_sentry() -> sentry::ClientInitGuard {
	// Use an empty string if we don't have any env variable for sentry. Sentry
	// will just silently ignore.
	let sentry = sentry::init(env::var("RCH_SENTRY_DSN").unwrap_or_else(|_| "".into()));
	if sentry.is_enabled() {
		info!(target: LOG_TARGET, "Sentry is successfully set up.")
	}

	sentry
}

/// If RCH_BACKEND_NAME environment variable is set, add it to the sentry
/// `server_name` properties.
/// For backwards compatibility, we also support HEROKU_APP_NAME env variable.
fn get_backend_name<'a>() -> Option<Cow<'a, str>> {
	if let Ok(n) = env::var("RCH_BACKEND_NAME") {
		return Some(n.into());
	} else if let Ok(n) = env::var("HEROKU_APP_NAME") {
		return Some(n.into());
	}

	None
}

#[derive(Debug)]
enum SentryError<'a> {
	// TODO: Probably a good idea would be to `impl std:error:Error` for the
	// three errors below.
	Misc(&'a MiscError),
	Mx(&'a MxError),
	Smtp(&'a SmtpError),
}

impl<'a> SentryError<'a> {
	/// Get the error type to be passed into Sentry's Exception `ty` field.
	fn get_exception_type(&self) -> String {
		match self {
			SentryError::Misc(_) => "MiscError".into(),
			SentryError::Mx(_) => "MxError".into(),
			SentryError::Smtp(_) => "SmtpError".into(),
		}
	}
}

/// Helper function to send an Error event to Sentry. We redact all sensitive
/// info before sending to Sentry, by removing all instances of `username`.
fn error(err: SentryError, result: &CheckEmailOutput) {
	let exception_value = redact(format!("{err:?}").as_str(), &result.syntax.username);
	debug!(target: LOG_TARGET, "Sending error to Sentry: {}", exception_value);

	let exception = Exception {
		ty: err.get_exception_type(),
		value: Some(exception_value),
		..Default::default()
	};

	sentry::capture_event(Event {
		exception: Values {
			values: vec![exception],
		},
		level: Level::Error,
		environment: Some("production".into()),
		release: Some(CARGO_PKG_VERSION.into()),
		message: Some(format!("{result:#?}")),
		server_name: get_backend_name(),
		transaction: Some(format!("check_email:{}", result.syntax.domain)),
		..Default::default()
	});
}

/// Function to replace all usernames from email, and replace them with
/// `***@domain.com` for privacy reasons.
fn redact(input: &str, username: &str) -> String {
	input.replace(username, "***")
}

/// Check if the message contains known SMTP Transient errors.
fn skip_smtp_transient_errors(message: &[String]) -> bool {
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
		(Err(err), _, _) => {
			// We log all misc errors.
			error(SentryError::Misc(err), result);
		}
		(_, Err(err), _) => {
			// We log all mx errors.
			error(SentryError::Mx(err), result);
		}
		(_, _, Err(err)) if err.get_description().is_some() => {
			// If the SMTP error is known, we don't track it in Sentry.
		}
		(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
			if skip_smtp_transient_errors(&response.message) =>
		{
			// If the SMTP error is transient and known, we don't track it in
			// Sentry, just log it locally.
			debug!(target: LOG_TARGET,
				"Transient error: {}",
				redact(
					response.message[0].as_str(),
					result.syntax.username.as_str()
				)
			);
		}
		(_, _, Err(err)) => {
			// If it's a SMTP error we didn't catch above, we log to
			// Sentry, to be able to debug them better. We don't want to
			// spam Sentry and log all instances of the error, hence the
			// `count` check.
			error(SentryError::Smtp(err), result);
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
