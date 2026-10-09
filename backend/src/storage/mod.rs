pub mod error;
pub mod postgres;

use crate::worker::do_work::{CheckEmailTask, TaskError};
use check_if_email_exists::CheckEmailOutput;
use error::StorageError;
use postgres::PostgresStorage;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub enum StorageAdapter {
	Postgres(PostgresStorage),
	#[default]
	Noop,
}

impl StorageAdapter {
	pub async fn store(
		&self,
		task: &CheckEmailTask,
		worker_output: &Result<CheckEmailOutput, TaskError>,
		extra: Option<serde_json::Value>,
	) -> Result<(), StorageError> {
		match self {
			StorageAdapter::Postgres(storage) => storage.store(task, worker_output, extra).await,
			StorageAdapter::Noop => Ok(()),
		}
	}

	pub fn get_extra(&self) -> Option<serde_json::Value> {
		match self {
			StorageAdapter::Postgres(storage) => storage.get_extra().clone(),
			StorageAdapter::Noop => None,
		}
	}
}
