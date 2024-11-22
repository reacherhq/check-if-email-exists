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

use check_if_email_exists::{CheckEmailInputBuilderError, LOG_TARGET};
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::fmt::Debug;
use tracing::error;
use warp::{http::StatusCode, reject};

/// Trait combining ToString and Debug.
pub trait ToStringDebug: ToString + Debug + Sync + Send {}

impl<T: ToString + Debug + Sync + Send> ToStringDebug for T {}

/// Struct describing an error response.
#[derive(Debug)]
pub struct ReacherResponseError {
	pub code: StatusCode,
	pub error: Box<dyn ToStringDebug>,
}

impl reject::Reject for ReacherResponseError {}

impl Serialize for ReacherResponseError {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("ReacherResponseError", 1)?;
		state.serialize_field("error", &self.error.to_string())?;
		state.end()
	}
}

impl ToString for ReacherResponseError {
	fn to_string(&self) -> String {
		self.error.to_string()
	}
}

impl ReacherResponseError {
	pub fn new<T: ToStringDebug + 'static>(code: StatusCode, error: T) -> Self {
		Self {
			code,
			error: Box::new(error),
		}
	}
}

impl From<CheckEmailInputBuilderError> for ReacherResponseError {
	fn from(e: CheckEmailInputBuilderError) -> Self {
		Self {
			code: StatusCode::BAD_REQUEST,
			error: Box::new(e),
		}
	}
}

impl From<serde_json::Error> for ReacherResponseError {
	fn from(e: serde_json::Error) -> Self {
		ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e)
	}
}

impl From<lapin::Error> for ReacherResponseError {
	fn from(e: lapin::Error) -> Self {
		ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e)
	}
}

impl From<sqlx::Error> for ReacherResponseError {
	fn from(e: sqlx::Error) -> Self {
		ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e)
	}
}

impl From<csv::Error> for ReacherResponseError {
	fn from(e: csv::Error) -> Self {
		ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e)
	}
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for ReacherResponseError {
	fn from(e: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
		ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e)
	}
}

/// This function receives a `Rejection` and tries to return a custom value,
/// otherwise simply passes the rejection along.
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
	if let Some(err) = err.find::<ReacherResponseError>() {
		error!(target: LOG_TARGET, code=?err.code, message=?err.to_string(), "Request rejected");
		Ok((warp::reply::with_status(warp::reply::json(err), err.code),))
	} else {
		Err(err)
	}
}
