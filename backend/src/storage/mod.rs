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

pub mod commercial_license_trial;
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
