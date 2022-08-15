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

use warp::reject;

#[derive(Debug)]
pub enum CsvError {
	CsvLib(csv::Error),
	CsvLibWriter(Box<csv::IntoInnerError<csv::Writer<Vec<u8>>>>),
	Parse(&'static str),
}

/// Catch all error struct for the bulk endpoints
#[derive(Debug)]
pub enum BulkError {
	EmptyInput,
	JobInProgress,
	Db(sqlx::Error),
	Csv(CsvError),
	Json(serde_json::Error),
}

// Defaults to Internal server error
impl reject::Reject for BulkError {}

// wrap sql errors as db errors for reacher
impl From<sqlx::Error> for BulkError {
	fn from(e: sqlx::Error) -> Self {
		BulkError::Db(e)
	}
}
