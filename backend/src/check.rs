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

use std::env;
use std::time::Duration;

use check_if_email_exists::{check_email as ciee_check_email, CheckEmailInput, CheckEmailOutput};
use warp::Filter;

use super::sentry_util;

/// Timeout after which we drop the `check-if-email-exists` check. We run the
/// checks twice (to avoid greylisting), so each verification takes 60s max.
const SMTP_TIMEOUT: u64 = 30;

/// Same as `check-if-email-exists`'s check email, but adds some additional
/// inputs and error handling.
pub async fn check_email(mut input: CheckEmailInput) -> CheckEmailOutput {
	input.set_smtp_timeout(Duration::from_secs(SMTP_TIMEOUT));
	input.set_hotmail_use_headless(
		env::var("RCH_HOTMAIL_USE_HEADLESS").unwrap_or_else(|_| "".into()) == "1",
	);

	let res = ciee_check_email(&input).await;

	sentry_util::log_unknown_errors(&res);

	res
}

/// The header which holds the Reacher backend secret.
pub const REACHER_SECRET_HEADER: &str = "x-reacher-secret";

/// Warp filter to check that the header secret is correct, if the environment
/// variable `RCH_HEADER_SECRET`  is set
pub fn check_header() -> warp::filters::BoxedFilter<()> {
	let env_var = env::var("RCH_HEADER_SECRET");

	match env_var {
		Ok(secret) => {
			let secret: &'static str = Box::leak(Box::new(secret));

			warp::header::exact("x-reacher-secret", secret).boxed()
		}
		Err(_) => warp::any().boxed(),
	}
}
