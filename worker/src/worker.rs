use check_if_email_exists::{check_email, CheckEmailOutput};
use lapin::message::Delivery;
use lapin::options::*;
use serde::Serialize;
use tracing::{debug, info};

use crate::CheckEmailPayload;

/// Represents a worker responsible for checking if an email exists.
pub struct CheckEmailWorker {
	client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct WebhookOutput {
	output: CheckEmailOutput,
	extra: serde_json::Value,
}

impl CheckEmailWorker {
	pub fn new() -> Self {
		Self {
			client: reqwest::Client::new(),
		}
	}

	pub async fn process_check_email(
		&self,
		delivery: Delivery,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let payload = serde_json::from_slice::<CheckEmailPayload>(&delivery.data)?;
		info!(email=?payload.input.to_email, "Start check");
		debug!(payload=?payload);

		let output = check_email(&payload.input).await;
		debug!(email=output.input,output=?output, "Done check-if-email-exists");

		if let Some(webhook) = payload.webhook {
			let webhook_output = WebhookOutput {
				output,
				extra: webhook.extra,
			};

			let res = self
				.client
				.post(webhook.url)
				.json(&webhook_output)
				.header("x-reacher-secret", std::env::var("RCH_HEADER_SECRET")?)
				.send()
				.await?
				.text()
				.await?;
			debug!(email=?webhook_output.output.input,res=?res, "Received webhook response");
		}

		delivery.ack(BasicAckOptions::default()).await?;

		info!(email=?payload.input.to_email, "Finished check");

		Ok(())
	}
}
