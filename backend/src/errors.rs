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

//! Describe a common response error to be used by all routes, should an error
//! happen.

use serde::Serialize;
use warp::{http, reject};

/// Struct describing an error response.
#[derive(Serialize, Debug)]
pub struct ReacherResponseError {
	#[serde(skip)]
	code: http::StatusCode,
	message: String,
}

impl reject::Reject for ReacherResponseError {}

/// This function receives a `Rejection` and tries to return a custom value,
/// otherwise simply passes the rejection along.
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
	if let Some(err) = err.find::<ReacherResponseError>() {
		Ok((warp::reply::with_status(warp::reply::json(err), err.code),))
	} else {
		Err(err)
	}
}
