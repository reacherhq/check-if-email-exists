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

use super::error::StorageError;
use super::postgres::PostgresStorage;
use super::Storage;
use crate::worker::do_work::{CheckEmailJobId, CheckEmailTask, TaskError};
use async_trait::async_trait;
use check_if_email_exists::{redact, CheckEmailOutput, LOG_TARGET};
use serde_json::Value;
use std::any::Any;
use tracing::debug;

/// Storage that's baked in the software for users of the Commercial License
/// trial. It's really just a wrapper around the PostgresStorage, where we
/// redact all sensitive data such as the email address.
#[derive(Debug)]
pub struct CommercialLicenseTrialStorage {
	postgres_storage: PostgresStorage,
}

impl CommercialLicenseTrialStorage {
	pub async fn new(db_url: &str, extra: Option<Value>) -> Result<Self, StorageError> {
		let postgres_storage = PostgresStorage::new(db_url, extra).await?;
		Ok(Self { postgres_storage })
	}
}

#[async_trait]
impl Storage for CommercialLicenseTrialStorage {
	async fn store(
		&self,
		task: &CheckEmailTask,
		worker_output: &Result<CheckEmailOutput, TaskError>,
		extra: Option<Value>,
	) -> Result<(), StorageError> {
		let mut payload_json = serde_json::to_value(task)?;
		if let Ok(output) = worker_output {
			redact_across_json(&mut payload_json, &output.syntax.username);
		}

		match worker_output {
			Ok(output) => {
				let mut output_json = serde_json::to_value(output)?;
				redact_across_json(&mut output_json, &output.syntax.username);

				sqlx::query!(
					r#"
					INSERT INTO v1_task_result (payload, job_id, extra, result)
					VALUES ($1, $2, $3, $4)
					RETURNING id
					"#,
					payload_json,
					match task.job_id {
						CheckEmailJobId::Bulk(job_id) => Some(job_id),
						CheckEmailJobId::SingleShot => None,
					},
					extra,
					output_json,
				)
				.fetch_one(&self.postgres_storage.pg_pool)
				.await?;
			}
			Err(err) => {
				sqlx::query!(
					r#"
					INSERT INTO v1_task_result (payload, job_id, extra, error)
					VALUES ($1, $2, $3, $4)
					RETURNING id
					"#,
					payload_json,
					match task.job_id {
						CheckEmailJobId::Bulk(job_id) => Some(job_id),
						CheckEmailJobId::SingleShot => None,
					},
					extra,
					err.to_string(),
				)
				.fetch_one(&self.postgres_storage.pg_pool)
				.await?;
			}
		}

		debug!(target: LOG_TARGET, email=?task.input.to_email, "Wrote to DB");

		Ok(())
	}

	fn get_extra(&self) -> Option<serde_json::Value> {
		self.postgres_storage.get_extra()
	}

	// This is a workaround to allow downcasting to Any, and should be removed
	// ref: https://github.com/reacherhq/check-if-email-exists/issues/1544
	fn as_any(&self) -> &dyn Any {
		self
	}
}

/// Redact all sensitive data by recursively traversing the JSON object.
fn redact_across_json(value: &mut Value, username: &str) {
	match value {
		Value::String(s) => *s = redact(s, username),
		Value::Array(arr) => {
			for item in arr {
				redact_across_json(item, username);
			}
		}
		Value::Object(obj) => {
			for (_, v) in obj {
				redact_across_json(v, username);
			}
		}
		_ => {}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use check_if_email_exists::{check_email, CheckEmailInputBuilder};

	#[tokio::test]
	async fn should_redact_across_json() {
		let input = CheckEmailInputBuilder::default()
			// Checking this email will make a MX record check, but hopefully
			// it won't resolve (since I typed it randomly), meaning that the
			// SMTP check will be skipped.
			.to_email("someone@adlkfjaklsdjfldksjfderlqkjeqwr.com".into())
			.build()
			.unwrap();
		let output = check_email(&input).await;
		let mut output_json = serde_json::to_value(&output).unwrap();
		redact_across_json(&mut output_json, &output.syntax.username);

		assert!(!output_json.to_string().contains("someone"));
	}
}
