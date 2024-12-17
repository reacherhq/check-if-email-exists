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

use crate::LOG_TARGET;
use pwned::api::PwnedBuilder;

/// Check if the email has been found in any breach or paste using the
/// HaveIBeenPwned API.
/// This function will return the number of times the email has been found in
/// any breach.
pub async fn check_haveibeenpwned(to_email: &str, api_key: Option<String>) -> Option<bool> {
	let pwned = PwnedBuilder::default()
		.user_agent("reacher")
		.api_key(api_key)
		.build()
		.unwrap();

	match pwned.check_email(to_email).await {
		Ok(answer) => {
			tracing::debug!(
				target: LOG_TARGET,
				breach_count=answer.len(),
				"HaveIBeenPwned check completed"
			);
			Some(!answer.is_empty())
		}
		Err(e) => {
			tracing::error!(
				target: LOG_TARGET,
				error=?e,
				"Error checking HaveIBeenPwned"
			);
			match e {
				pwned::errors::Error::IoError(e) => match e.kind() {
					std::io::ErrorKind::NotFound => Some(false),
					_ => None,
				},
				_ => None,
			}
		}
	}
}
