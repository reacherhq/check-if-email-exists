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

//! This file implements the `POST /v0/check_email` endpoint.

use check_if_email_exists::{check_email, CheckEmailInput, LOG_TARGET};
use warp::{http, Filter};

use crate::check::check_header;
use crate::errors;

/// The main endpoint handler that implements the logic of this route.
async fn handler(body: CheckEmailInput) -> Result<impl warp::Reply, warp::Rejection> {
	// The to_email field must be present
	if body.to_email.is_empty() {
		Err(warp::reject::custom(errors::ReacherResponseError {
			code: http::StatusCode::BAD_REQUEST,
			message: "to_email field is required.".to_string(),
		}))
	} else {
		// Run the future to check an email.
		Ok(warp::reply::json(&check_email(&body).await))
	}
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v0" / "check_email")
		.and(warp::post())
		.and(check_header())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(handler)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log(LOG_TARGET))
		.recover(errors::handle_rejection)
}
