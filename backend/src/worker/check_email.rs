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

use check_if_email_exists::CheckEmailInput;
use check_if_email_exists::LOG_TARGET;
use lapin::message::Delivery;
use lapin::{options::*, BasicProperties, Channel};
use serde::Deserialize;
use tracing::{debug, info};

use crate::check::check_email;

#[derive(Debug, Deserialize)]
pub struct CheckEmailPayload {
	pub input: CheckEmailInput,
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
	debug!(target: LOG_TARGET, email=output.input,output=?output, "Done check-if-email-exists");

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
	}

	delivery.ack(BasicAckOptions::default()).await?;

	Ok(())
}
