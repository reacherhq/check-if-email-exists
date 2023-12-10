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

use std::env;

use check_if_email_exists::CheckEmailInput;
use check_if_email_exists::{check_email as ciee_check_email, CheckEmailOutput};
use lapin::message::Delivery;
use lapin::options::*;
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, info};

use crate::sentry_util::log_unknown_errors;

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
	info!(email=?payload.input.to_email, "Start check");
	debug!(payload=?payload);

	let output = check_email(payload.input).await;
	debug!(email=output.input,output=?output, "Done check-if-email-exists");

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
		debug!(email=?webhook_output.output.input,res=?res, "Received webhook response");
		info!(email=?webhook_output.output.input, "Finished check");
	}

	delivery.ack(BasicAckOptions::default()).await?;

	Ok(())
}

/// Same as `check-if-email-exists`'s check email, but adds some additional
/// inputs and error handling.
async fn check_email(input: CheckEmailInput) -> CheckEmailOutput {
	let from_email =
		env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| CheckEmailInput::default().from_email);
	let hello_name: String =
		env::var("RCH_HELLO_NAME").unwrap_or_else(|_| CheckEmailInput::default().hello_name);

	let input = CheckEmailInput {
		// If we want to override core check-if-email-exists's default values
		// for CheckEmailInput for the backend, we do it here.
		from_email,
		hello_name,
		..input
	};

	let res = ciee_check_email(&input).await;

	log_unknown_errors(&res);

	res
}
