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

use crate::util::input_output::CheckEmailInput;
use reqwest::Error as ReqwestError;

/// Helper function to create a reqwest client, with optional proxy.
pub fn create_client(
	_input: &CheckEmailInput,
	_api_name: &str,
) -> Result<reqwest::Client, ReqwestError> {
	// TODO: Allow proxying for HTTP API requests.
	Ok(reqwest::Client::new())
}
