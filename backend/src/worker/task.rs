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

use super::db::save_to_db;
use crate::config::{BackendConfig, Queue};
use check_if_email_exists::mx::check_mx;
use check_if_email_exists::syntax::check_syntax;
use check_if_email_exists::{
	check_email, is_gmail, is_hotmail_b2b, is_hotmail_b2c, is_yahoo, CheckEmailInput,
	CheckEmailOutput, HotmailVerifMethod, Reachable, YahooVerifMethod, LOG_TARGET,
};
use core::time;
use lapin::message::Delivery;
use lapin::{options::*, BasicProperties, Channel};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};
use warp::http::StatusCode;

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskPayload {
	pub input: CheckEmailInput,
	// If the task is a part of a job, then this field will be set.
	pub job_id: Option<i32>,
	pub webhook: Option<TaskWebhook>,
}

impl TaskPayload {
	/// Returns true if the task is a single-shot email verification via the
	/// /v1/check_email, endpoint, i.e. not a part of a bulk verification job.
	pub fn is_single_shot(&self) -> bool {
		self.job_id.is_none()
	}
}

/// The errors that can occur when processing a task.
#[derive(Debug, Error)]
pub enum TaskError {
	/// The worker is at full capacity and cannot accept more tasks. Note that
	/// this error only occurs for single-shot tasks, and not for bulk
	/// verification, as for bulk verification tasks the task will simply stay
	/// in the queue until one worker is ready to process it.
	#[error("Worker at full capacity, wait {0:?}")]
	Throttle(time::Duration),
	#[error("Lapin error: {0}")]
	Lapin(lapin::Error),
	#[error("Reqwest error during webhook: {0}")]
	Reqwest(reqwest::Error),
}

impl TaskError {
	/// Returns the status code that should be returned to the client.
	pub fn status_code(&self) -> StatusCode {
		match self {
			Self::Throttle(_) => StatusCode::TOO_MANY_REQUESTS,
			Self::Lapin(_) => StatusCode::INTERNAL_SERVER_ERROR,
			Self::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

impl From<lapin::Error> for TaskError {
	fn from(err: lapin::Error) -> Self {
		Self::Lapin(err)
	}
}

impl From<reqwest::Error> for TaskError {
	fn from(err: reqwest::Error) -> Self {
		Self::Reqwest(err)
	}
}

impl Serialize for TaskError {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

/// For single-shot email verifications, the worker will send a reply to the
/// client with the result of the verification.
#[derive(Debug, Deserialize, Serialize)]
pub struct SingleShotResponse {
	/// The HTTP status code to send to the client.
	pub code: u16, // Unfortunately we can't use warp::http::StatusCode here.
	/// The JSON-encoded result to send to the client. It can be either a
	/// CheckEmailOutput or a TaskError, serialized to JSON. Because a lot of
	/// error types don't implement Deserialize, we use Vec<u8> here.
	pub body: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TaskWebhook {
	pub on_each_email: Option<TaskWebhookOnEachEmail>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TaskWebhookOnEachEmail {
	pub url: String,
	pub extra: Option<serde_json::Value>,
}

impl TryFrom<Result<CheckEmailOutput, TaskError>> for SingleShotResponse {
	type Error = serde_json::Error;

	fn try_from(result: Result<CheckEmailOutput, TaskError>) -> Result<Self, Self::Error> {
		match result {
			Ok(output) => Ok(Self {
				code: StatusCode::OK.as_u16(),
				body: serde_json::to_vec(&output)?,
			}),
			Err(err) => Ok(Self {
				code: err.status_code().as_u16(),
				body: serde_json::to_vec(&err)?,
			}),
		}
	}
}

#[derive(Debug, Serialize)]
struct WebhookOutput<'a> {
	result: &'a CheckEmailOutput,
	extra: &'a Option<serde_json::Value>,
}

/// Processes the check email task asynchronously.
pub(crate) async fn process_queue_message(
	payload: &TaskPayload,
	delivery: Delivery,
	channel: Arc<Channel>,
	pg_pool: PgPool,
	config: Arc<BackendConfig>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let worker_output = inner_process_queue_message(payload, &delivery, Arc::clone(&config)).await;

	if payload.is_single_shot() {
		// Send reply by following this guide:
		// https://www.rabbitmq.com/tutorials/tutorial-six-javascript.html
		//
		// This only applies for single-shot email verifications on the
		// /v1/check_email endpoint, and not to bulk verifications.
		if let (Some(reply_to), Some(correlation_id)) = (
			delivery.properties.reply_to(),
			delivery.properties.correlation_id(),
		) {
			let properties = BasicProperties::default()
				.with_correlation_id(correlation_id.to_owned())
				.with_content_type("application/json".into());

			let single_shot_response = SingleShotResponse::try_from(worker_output)?;
			let reply_payload = serde_json::to_vec(&single_shot_response)?;

			channel
				.basic_publish(
					"",
					reply_to.as_str(),
					BasicPublishOptions::default(),
					&reply_payload,
					properties,
				)
				.await?
				.await?;

			debug!(target: LOG_TARGET, reply_to=?reply_to.to_string(), correlation_id=?correlation_id.to_string(), "Sent reply")
		} else {
			return Err("Missing reply_to or correlation_id".into());
		}
	} else {
		save_to_db(&config.backend_name, pg_pool, payload, worker_output).await?;
	}

	Ok(())
}

async fn inner_process_queue_message(
	payload: &TaskPayload,
	delivery: &Delivery,
	config: Arc<BackendConfig>,
) -> Result<CheckEmailOutput, TaskError> {
	let output = check_email(&payload.input, &config.get_reacher_config()).await;

	// Check if we have a webhook to send the output to.
	if let Some(TaskWebhook {
		on_each_email: Some(webhook),
	}) = &payload.webhook
	{
		let webhook_output = WebhookOutput {
			result: &output,
			extra: &webhook.extra,
		};

		let client = reqwest::Client::new();
		let res = client
			.post(&webhook.url)
			.json(&webhook_output)
			.header(
				"x-reacher-secret",
				std::env::var("RCH_HEADER_SECRET").unwrap_or("".into()),
			)
			.send()
			.await?
			.text()
			.await?;
		debug!(target: LOG_TARGET, email=?webhook_output.result.input,res=?res, "Received webhook response");
	}

	let is_reachable = output.is_reachable.to_owned();

	// If is_reachable is unknown, then we requeue the message, but only once.
	// We might want to add a requeue counter in the future, see:
	// https://stackoverflow.com/questions/25226080/rabbitmq-how-to-requeue-message-with-counter
	if is_reachable == Reachable::Unknown && !delivery.redelivered {
		delivery
			.reject(BasicRejectOptions { requeue: true })
			.await?;
		info!(target: LOG_TARGET, email=?&payload.input.to_email, "Requeued message");
	} else {
		delivery.ack(BasicAckOptions::default()).await?;

		info!(target: LOG_TARGET, email=output.input, is_reachable=?output.is_reachable, job_id=?payload.job_id, "Done check");
	}

	Ok(output)
}

pub async fn preprocess(
	payload: &TaskPayload,
	delivery: Delivery,
	channel: Arc<Channel>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let syntax = check_syntax(&payload.input.to_email);
	let mx = check_mx(&syntax).await?;
	// Get first hostname from MX records.
	let mx_hostname = mx
		.lookup?
		.iter()
		.next()
		.ok_or_else(|| "No MX records found")?
		.exchange()
		.to_string();

	let queue = if is_gmail(&mx_hostname) {
		Queue::GmailSmtp
	} else if is_hotmail_b2b(&mx_hostname) {
		Queue::HotmailB2BSmtp
	} else if is_hotmail_b2c(&mx_hostname) {
		if payload.input.hotmail_verif_method == HotmailVerifMethod::Smtp {
			Queue::HotmailB2CSmtp
		} else {
			Queue::HotmailB2CHeadless
		}
	} else if is_yahoo(&mx_hostname) {
		if payload.input.yahoo_verif_method == YahooVerifMethod::Smtp {
			Queue::YahooSmtp
		} else {
			Queue::YahooHeadless
		}
	} else {
		Queue::EverythingElseSmtp
	};

	channel
		.basic_publish(
			"",
			format!("{}", queue).as_str(),
			BasicPublishOptions::default(),
			&delivery.data,
			delivery.properties.clone(),
		)
		.await?
		.await?;

	delivery.ack(BasicAckOptions::default()).await?;
	debug!(target: LOG_TARGET, email=?payload.input.to_email, queue=?queue.to_string(), "Message preprocessed");

	Ok(())
}
