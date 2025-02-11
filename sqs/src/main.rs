// check-if-email-exists
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

use check_if_email_exists::CheckEmailOutput;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use reacher_backend::config::{load_config, BackendConfig};
use reacher_backend::http::CheckEmailRequest;
use reacher_backend::storage::commercial_license_trial::send_to_reacher;
use reacher_backend::worker::do_work::{
	check_email_and_send_result, CheckEmailJobId, CheckEmailTask, TaskWebhook,
};
use serde::Deserialize;
use std::process::Command;
use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// Too bad aws_sdk_sqs::types::Message is not Deserialize, now we need to
// create our own struct to deserialize the message.
#[derive(Debug, Deserialize)]
struct SQSMessage {
	body: String,
}

/// The payload of the SQS event that's passed into the lambda.
#[derive(Debug, Deserialize)]
struct SQSPayload {
	#[serde(rename = "Records")]
	records: Vec<SQSMessage>,
}

/// This is like CheckEmailTask, but where the input is a CheckEmailRequest
/// instead of a CheckEmailInput. We can transform the CheckEmailRequest into
/// a CheckEmailInput by calling to_check_email_input.
#[derive(Debug, Deserialize)]
struct CheckEmailPartialTask {
	input: CheckEmailRequest,
	job_id: CheckEmailJobId,
	webhook: Option<TaskWebhook>,
}

impl CheckEmailPartialTask {
	fn into_check_email_task(self, backend_config: Arc<BackendConfig>) -> CheckEmailTask {
		CheckEmailTask {
			input: self.input.to_check_email_input(backend_config),
			job_id: self.job_id,
			webhook: self.webhook,
		}
	}
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	tracing_subscriber::fmt()
		// Setting a filter based on the value of the `RUST_LOG` environment
		.with_env_filter(EnvFilter::from_default_env())
		.json()
		// this needs to be set to remove duplicated information in the log.
		.with_current_span(false)
		// this needs to be set to false, otherwise ANSI color codes will
		// show up in a confusing manner in CloudWatch logs.
		.with_ansi(false)
		// disabling time is handy because CloudWatch will add the ingestion time.
		.without_time()
		// remove the name of the function from every log entry
		.with_target(false)
		.init();
	info!(version=?CARGO_PKG_VERSION, "Starting Reacher SQS lambda.");

	run_and_wait_chromedriver().await?;

	lambda_runtime::run(service_fn(handler)).await?;
	Ok(())
}

async fn handler(event: LambdaEvent<SQSPayload>) -> Result<CheckEmailOutput, Error> {
	let (request, _context) = event.into_parts();
	// Since we're only fetching a single message, we can safely unwrap here.
	let message = request.records.first().expect("No messages in the event");
	let task: CheckEmailPartialTask = serde_json::from_str(&message.body)?;
	info!(email = ?task.input.to_email, "Processing task");

	let backend_config = Arc::new(load_config().await?);
	debug!("{:#?}", backend_config);

	let task = &task.into_check_email_task(backend_config.clone());

	let worker_output = check_email_and_send_result(task).await;
	match worker_output.as_ref() {
		Ok(output) => {
			info!(email = ?output.input, is_reachable = ?output.is_reachable, "Task completed");
		}
		Err(e) => {
			info!(email = ?task.input.to_email, err = ?e, "Task failed");
		}
	}

	// TODO:
	// - Refactor storing the result and sending to Reacher, it's duplicated
	// code from the backend.
	// - Add throttling, again using backend code.

	// Store the result.
	let storage = backend_config.get_storage_adapter();
	storage
		.store(task, &worker_output, storage.get_extra())
		.await?;

	// If we're in the Commercial License Trial, we also store the
	// result by sending it to back to Reacher.
	send_to_reacher(backend_config, &task.input.to_email, &worker_output).await?;

	Ok(worker_output?)
}

async fn run_and_wait_chromedriver() -> Result<(), Error> {
	Command::new("/opt/chromedriver-linux64/chromedriver")
		.arg("--port=9515")
		.spawn()?;

	// Wait until the chromedriver is ready.
	let mut attempts = 0;
	let max_attempts = 10;
	let delay = std::time::Duration::from_secs(1);

	while attempts < max_attempts {
		if let Ok(output) = reqwest::get("http://localhost:9515/status").await {
			if output.status().is_success() {
				info!("Chromedriver is ready.");
				break;
			}
		}
		attempts += 1;
		tokio::time::sleep(delay).await;
	}

	if attempts == max_attempts {
		return Err(Error::from("Chromedriver did not start in time"));
	}

	Ok(())
}
