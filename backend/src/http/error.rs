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

use check_if_email_exists::LOG_TARGET;
use serde::Serialize;
use tracing::error;
use warp::{http, reject};

/// Struct describing an error response.
#[derive(Serialize, Debug)]
pub struct ReacherResponseError {
	#[serde(skip)]
	pub code: http::StatusCode,
	pub message: String,
}

impl ReacherResponseError {
	pub fn new(code: http::StatusCode, message: String) -> Self {
		Self {
			code,
			message: message,
		}
	}
}

impl reject::Reject for ReacherResponseError {}

/// This function receives a `Rejection` and tries to return a custom value,
/// otherwise simply passes the rejection along.
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
	if let Some(err) = err.find::<ReacherResponseError>() {
		error!(target: LOG_TARGET, code=?err.code, message=?err.message, "Request rejected");
		Ok((warp::reply::with_status(warp::reply::json(err), err.code),))
	} else {
		Err(err)
	}
}
