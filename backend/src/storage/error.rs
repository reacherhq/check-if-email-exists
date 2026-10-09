use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
	#[error("SQLX error: {0}")]
	SqlxError(#[from] sqlx::error::Error),
	#[error("SQLX migrate error: {0}")]
	MigrateError(#[from] sqlx::migrate::MigrateError),
	#[error("Serde JSON error: {0}")]
	SerdeJsonError(#[from] serde_json::Error),
}
