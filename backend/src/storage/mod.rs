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
use async_trait::async_trait;
use check_if_email_exists::CheckEmailOutput;
use error::StorageError;
use std::any::Any;
use std::fmt::Debug;

#[async_trait]
pub trait Storage: Debug + Send + Sync + Any {
	async fn store(
		&self,
		task: &CheckEmailTask,
		worker_output: &Result<CheckEmailOutput, TaskError>,
		extra: Option<serde_json::Value>,
	) -> Result<(), StorageError>;

	fn get_extra(&self) -> Option<serde_json::Value>;

	// This is a workaround to allow downcasting to Any, and should be removed
	// ref: https://github.com/reacherhq/check-if-email-exists/issues/1544
	fn as_any(&self) -> &dyn Any;
}
