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
