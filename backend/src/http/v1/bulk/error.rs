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

use warp::reject;

/// Catch all error struct for the bulk endpoints
#[derive(Debug)]
pub enum BulkError {
	EmptyInput,
	Serde(serde_json::Error),
	Lapin(lapin::Error),
}

// Defaults to Internal server error
impl reject::Reject for BulkError {}

impl From<serde_json::Error> for BulkError {
	fn from(e: serde_json::Error) -> Self {
		BulkError::Serde(e)
	}
}

impl From<lapin::Error> for BulkError {
	fn from(e: lapin::Error) -> Self {
		BulkError::Lapin(e)
	}
}
