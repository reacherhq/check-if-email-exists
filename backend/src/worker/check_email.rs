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

use check_if_email_exists::{CheckEmailInput, CheckEmailOutput};
use check_if_email_exists::{Reachable, LOG_TARGET};
use lapin::message::Delivery;
use lapin::{options::*, BasicProperties, Channel};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::check::check_email;

#[derive(Debug, Deserialize)]
pub struct CheckEmailPayload {
	pub input: CheckEmailInput,
	pub webhook: Option<CheckEmailWebhook>,
}

#[derive(Debug, Deserialize)]
pub struct CheckEmailWebhook {
	pub url: String,
	pub extra: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct WebhookOutput {
	output: CheckEmailOutput,
	extra: serde_json::Value,
}

/// Processes the check email task asynchronously.
pub async fn process_check_email(
	channel: &Channel,
	delivery: Delivery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let payload = serde_json::from_slice::<CheckEmailPayload>(&delivery.data)?;
	info!(target: LOG_TARGET, email=?payload.input.to_email, "New job");
	debug!(target: LOG_TARGET, payload=?payload);

	let output = check_email(payload.input).await;
	info!(target: LOG_TARGET, email=output.input, is_reachable=?output.is_reachable, "Done check");
	debug!(target: LOG_TARGET, output=?output, "Done check");
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

		debug!(target: LOG_TARGET, reply_to=?reply_to, correlation_id=?correlation_id,  "Sent reply")
	}

	let (email, is_reachable) = (output.input.to_owned(), output.is_reachable.clone());

	// Check if we have a webhook to send the output to.
	if let Some(webhook) = payload.webhook {
		let webhook_output = WebhookOutput {
			output,
			extra: webhook.extra,
		};

		let client = reqwest::Client::new();
		let res = client
			.post(webhook.url)
			.json(&webhook_output)
			.header("x-reacher-secret", std::env::var("RCH_HEADER_SECRET")?)
			.send()
			.await?
			.text()
			.await?;
		debug!(target: LOG_TARGET, email=?webhook_output.output.input,res=?res, "Received webhook response");
		info!(target: LOG_TARGET, email=?webhook_output.output.input, is_reachable=?webhook_output.output.is_reachable, "Finished check");
	}

	// If is_reachable is unknown, then we requeue the message, but only once.
	// We might want to add a requeue counter in the future, see:
	// https://stackoverflow.com/questions/25226080/rabbitmq-how-to-requeue-message-with-counter
	if is_reachable == Reachable::Unknown && !delivery.redelivered {
		delivery
			.reject(BasicRejectOptions { requeue: true })
			.await?;
		info!(target: LOG_TARGET, email=?email, "Requeued message");
	} else {
		delivery.ack(BasicAckOptions::default()).await?;
	}

	Ok(())
}
