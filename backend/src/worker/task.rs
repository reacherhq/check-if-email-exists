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

use crate::config::BackendConfig;

use super::db::save_to_db;
use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailOutput};
use check_if_email_exists::{Reachable, LOG_TARGET};
use lapin::message::Delivery;
use lapin::{options::*, BasicProperties, Channel};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskPayload {
	pub input: CheckEmailInput,
	pub webhook: Option<TaskWebhook>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TaskWebhook {
	pub url: String,
	pub extra: Option<serde_json::Value>,
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
	let worker_output =
		process_queue_message_inner(&payload, delivery, channel, config.clone()).await;
	save_to_db(&config.backend_name, pg_pool, payload, worker_output).await
}

async fn process_queue_message_inner(
	payload: &TaskPayload,
	delivery: Delivery,
	channel: Arc<Channel>,
	config: Arc<BackendConfig>,
) -> Result<CheckEmailOutput, Box<dyn std::error::Error + Send + Sync>> {
	info!(target: LOG_TARGET, email=payload.input.to_email, "Start email verification");

	let output = check_email(&payload.input, &config.get_reacher_config()).await;
	let reply_payload = serde_json::to_string(&output)?;
	let reply_payload = reply_payload.as_bytes();

	// Send reply by following this guide:
	// https://www.rabbitmq.com/tutorials/tutorial-six-javascript.html
	if let (Some(reply_to), Some(correlation_id)) = (
		delivery.properties.reply_to(),
		delivery.properties.correlation_id(),
	) {
		let properties = BasicProperties::default()
			.with_correlation_id(correlation_id.to_owned())
			.with_content_type("application/json".into());

		channel
			.basic_publish(
				"",
				reply_to.as_str(),
				BasicPublishOptions::default(),
				reply_payload,
				properties,
			)
			.await?
			.await?;

		debug!(target: LOG_TARGET, reply_to=?reply_to, correlation_id=?correlation_id, "Sent reply")
	}

	// Check if we have a webhook to send the output to.
	if let Some(webhook) = &payload.webhook {
		let webhook_output = WebhookOutput {
			result: &output,
			extra: &webhook.extra,
		};

		let client = reqwest::Client::new();
		let res = client
			.post(&webhook.url)
			.json(&webhook_output)
			.header("x-reacher-secret", std::env::var("RCH_HEADER_SECRET")?)
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
		info!(target: LOG_TARGET, email=output.input, is_reachable=?output.is_reachable, "Done check");
	}

	Ok(output)
}