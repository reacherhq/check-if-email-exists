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

use check_if_email_exists::LOG_TARGET;
use check_if_email_exists::{CheckEmailInput, CheckEmailOutput};
use lapin::message::Delivery;
use lapin::options::*;
use serde::Deserialize;
use serde::Serialize;
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
	delivery: Delivery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let payload = serde_json::from_slice::<CheckEmailPayload>(&delivery.data)?;
	info!(target: LOG_TARGET, email=?payload.input.to_email, "New job");
	debug!(target: LOG_TARGET, payload=?payload);

	let output = check_email(payload.input).await;
	debug!(target: LOG_TARGET, email=output.input,output=?output, "Done check-if-email-exists");

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

	delivery.ack(BasicAckOptions::default()).await?;

	Ok(())
}
