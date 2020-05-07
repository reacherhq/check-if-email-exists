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

use serde::Serialize;

/// Details that we gathered from connecting to this email via SMTP.
#[derive(Debug, Serialize)]
pub struct MiscDetails {
	/// Is this a DEA (disposable email account)?
	pub is_disposable: bool,
}

impl Default for MiscDetails {
	fn default() -> Self {
		MiscDetails {
			is_disposable: false,
		}
	}
}

/// Error occured connecting to this email server via SMTP.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum MiscError {
	/// Skipped checking SMTP details.
	Skipped,
}

/// Fetch misc details about the email address, such as whether it's disposable.
pub fn check_misc(address: &str) -> MiscDetails {
	MiscDetails {
		// mailchecker::is_valid checks also if the syntax is valid. But if
		// we're here, it means we're sure the syntax is valid, so is_valid
		// actually will only check if it's disposable.
		is_disposable: !mailchecker::is_valid(address),
	}
}
