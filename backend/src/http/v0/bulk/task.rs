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

//! This file implements the `POST /bulk` endpoint.

use check_if_email_exists::{
	check_email, CheckEmailInput, CheckEmailInputProxy, CheckEmailOutput, Reachable, LOG_TARGET,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use sqlxmq::{job, CurrentJob};
use std::error::Error;
use tracing::{debug, error};
use uuid::Uuid;

use super::error::BulkError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskInput {
	// fields for CheckEmailInput
	pub to_email: String,     // Email from request to verify.
	pub smtp_ports: Vec<u16>, // Ports to try for each email, in given order. Defaults to [25].
	pub proxy: Option<CheckEmailInputProxy>,
	pub hello_name: Option<String>,
	pub from_email: Option<String>,
}

pub struct TaskInputIterator {
	body: TaskInput,
	index: usize,
}

impl IntoIterator for TaskInput {
	type Item = CheckEmailInput;
	type IntoIter = TaskInputIterator;

	fn into_iter(self) -> Self::IntoIter {
		TaskInputIterator {
			body: self,
			index: 0,
		}
	}
}

/// Iterate through all the `smtp_ports`.
impl Iterator for TaskInputIterator {
	type Item = CheckEmailInput;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.body.smtp_ports.len() {
			let mut item = CheckEmailInput::new(self.body.to_email.clone());

			if let Some(name) = &self.body.hello_name {
				item.set_hello_name(name.clone());
			}

			if let Some(email) = &self.body.from_email {
				item.set_from_email(email.clone());
			}

			item.set_smtp_port(self.body.smtp_ports[self.index]);

			if let Some(proxy) = &self.body.proxy {
				item.set_proxy(proxy.clone());
			}

			self.index += 1;
			Some(item)
		} else {
			None
		}
	}
}

/// Struct that's serialized into the sqlxmq own `payload_json` table.
#[derive(Debug, Deserialize, Serialize)]
struct TaskPayload {
	id: i32,
	input: TaskInput,
}

pub async fn submit_job(
	conn_pool: &Pool<Postgres>,
	job_id: i32,
	task_input: TaskInput,
) -> Result<Uuid, BulkError> {
	let task_payload = TaskPayload {
		id: job_id,
		input: task_input,
	};

	let uuid = email_verification_task
		.builder()
		.set_json(&task_payload)
		.map_err(|e| {
			error!(
				target: LOG_TARGET,
				"Failed to submit task with the following [input={:?}] with [error={}]",
				task_payload.input, e
			);

			BulkError::Json(e)
		})?
		.spawn(conn_pool)
		.await
		.map_err(|e| {
			error!(
				target: LOG_TARGET,
				"Failed to submit task for [bulk_req={}] with [error={}]",
				job_id, e
			);

			e
		})?;

	Ok(uuid)
}

/// Arguments to the `#[job]` attribute allow setting default task options.
/// This task tries to verify the given email and inserts the results
/// into the email verification db table
/// NOTE: if EMAIL_TASK_BATCH_SIZE is made greater than 1 this logic
/// will have to be changed to handle a vector outputs from `check_email`.
///
/// Small note about namings: what sqlxmq calls a "job", we call it a "task".
/// We call a "job" a user bulk request, i.e. a list of "tasks".
/// Please be careful while reading code.
#[job]
pub async fn email_verification_task(
	mut current_job: CurrentJob,
	// Additional arguments are optional, but can be used to access context
	// provided via [`JobRegistry::set_context`].
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	let task_payload: TaskPayload = current_job.json()?.ok_or("Got empty task.")?;
	let job_id = task_payload.id;

	let mut final_response: Option<CheckEmailOutput> = None;

	for check_email_input in task_payload.input {
		debug!(
			target: LOG_TARGET,
			"Starting task [email={}] for [job={}] and [uuid={}]",
			check_email_input.to_email,
			task_payload.id,
			current_job.id(),
		);

		let to_email = check_email_input.to_email.clone();
		let response = check_email(&check_email_input).await;

		debug!(
			target: LOG_TARGET,
			"Got task result [email={}] for [job={}] and [uuid={}] with [is_reachable={:?}]",
			to_email,
			task_payload.id,
			current_job.id(),
			response.is_reachable,
		);

		let is_reachable = response.is_reachable == Reachable::Unknown;
		final_response = Some(response);
		// unsuccessful validation continue iteration with next possible smtp port
		if is_reachable {
			continue;
		}
		// successful validation attempt complete job break iteration
		else {
			break;
		}
	}

	// final response can only be empty if there
	// were no validation attempts. This can can
	// never occur currently
	if let Some(response) = final_response {
		// write results and terminate iteration
		#[allow(unused_variables)]
		let rec = sqlx::query!(
			r#"
			INSERT INTO email_results (job_id, result)
			VALUES ($1, $2)
			"#,
			job_id,
			serde_json::json!(response)
		)
		// TODO: This is a simplified solution and will work when
		// the job queue and email results tables are in the same
		// database. Keeping them in separate database will require
		// some custom logic on the job registry side
		// https://github.com/Diggsey/sqlxmq/issues/4
		.fetch_optional(current_job.pool())
		.await
		.map_err(|e| {
			error!(
				target: LOG_TARGET,
				"Failed to write [email={}] result to db for [job={}] and [uuid={}] with [error={}]",
				response.input,
				job_id,
				current_job.id(),
				e
			);

			e
		})?;

		debug!(
			target: LOG_TARGET,
			"Wrote result for [email={}] for [job={}] and [uuid={}]",
			response.input,
			job_id,
			current_job.id(),
		);
	}

	current_job.complete().await?;
	Ok(())
}
