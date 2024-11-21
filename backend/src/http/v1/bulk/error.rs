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

use crate::http::error::ReacherResponseError;
use check_if_email_exists::CheckEmailInputBuilderError;
use warp::http::StatusCode;

/// Catch all error struct for the bulk endpoints
#[derive(Debug)]
pub enum BulkError {
	EmptyInput,
	JobInProgress,
	InputError(CheckEmailInputBuilderError),
	Serde(serde_json::Error),
	Lapin(lapin::Error),
	Sqlx(sqlx::Error),
	Csv(CsvError),
}

impl From<&BulkError> for ReacherResponseError {
	fn from(value: &BulkError) -> Self {
		match value {
			BulkError::EmptyInput => {
				ReacherResponseError::new(StatusCode::BAD_REQUEST, "Empty input".to_string())
			}
			BulkError::JobInProgress => {
				ReacherResponseError::new(StatusCode::BAD_REQUEST, "Job in progress".to_string())
			}
			BulkError::InputError(e) => {
				ReacherResponseError::new(StatusCode::BAD_REQUEST, e.to_string())
			}
			BulkError::Serde(e) => {
				ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
			}
			BulkError::Lapin(e) => {
				ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
			}
			BulkError::Sqlx(e) => {
				ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
			}
			BulkError::Csv(e) => {
				ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
			}
		}
	}
}

impl warp::reject::Reject for BulkError {}

impl From<CheckEmailInputBuilderError> for BulkError {
	fn from(e: CheckEmailInputBuilderError) -> Self {
		BulkError::InputError(e)
	}
}

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

impl From<sqlx::Error> for BulkError {
	fn from(e: sqlx::Error) -> Self {
		BulkError::Sqlx(e)
	}
}

/// Handle warp rejections for /v1/bulk endpoints
pub async fn v1_bulk_handle_rejection(
	err: warp::Rejection,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
	if let Some(err) = err.find::<BulkError>() {
		Err(warp::reject::custom(ReacherResponseError::from(err)))
	} else {
		Err(err)
	}
}

#[derive(Debug)]
pub enum CsvError {
	CsvLib(csv::Error),
	CsvLibWriter(Box<csv::IntoInnerError<csv::Writer<Vec<u8>>>>),
	Parse(&'static str),
}
