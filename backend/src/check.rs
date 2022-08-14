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

//! This file contains shared logic for checking one email.

use super::sentry_util;
use check_if_email_exists::{check_email as ciee_check_email, CheckEmailInput, CheckEmailOutput};
use std::time::Instant;

/// Timeout after which we drop the `check-if-email-exists` check. We run the
/// checks twice (to avoid greylisting), so each verification takes 20s max.
pub const SMTP_TIMEOUT: u64 = 10;

/// Same as `check-if-email-exists`'s check email, but adds some additional
/// logging and error handling, and also only handles 1 email.
///
/// # Panics
///
/// If more than 1 email is passed inside input, then this function panics.
pub async fn check_email(input: &CheckEmailInput) -> CheckEmailOutput {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time.
	let now = Instant::now();

	assert!(
		input.to_emails.len() == 1,
		"We currently hardcode the BATCH_SIZE to 1. qed."
	);

	let res = ciee_check_email(input)
		.await
		.pop()
		.expect("Input only has one email, so does output. qed.");

	// Log on Sentry the `is_reachable` field.
	// We should definitely log this somewhere else than Sentry.
	// TODO https://github.com/reacherhq/backend/issues/207
	sentry_util::metrics(
		format!("is_reachable={:?}", res.is_reachable),
		now.elapsed().as_millis(),
		res.syntax.domain.as_ref(),
	);

	sentry_util::log_unknown_errors(&res);

	res
}
