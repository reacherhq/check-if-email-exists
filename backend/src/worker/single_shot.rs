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

use super::do_work::TaskError;
use anyhow::bail;
use check_if_email_exists::{CheckEmailOutput, LOG_TARGET};
use lapin::message::Delivery;
use lapin::options::BasicPublishOptions;
use lapin::{BasicProperties, Channel};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::Arc;
use tracing::debug;
use warp::http::StatusCode;

/// For single-shot email verifications, the worker will send a reply to the
/// client with the result of the verification. Since both CheckEmailOutput and
/// TaskError are not Deserialize, we need to create a new struct that can be
/// serialized and deserialized.
#[derive(Debug, Deserialize, Serialize)]
pub enum SingleShotReply {
	/// JSON serialization of CheckEmailOutput
	Ok(Vec<u8>),
	/// String representation of TaskError with its status code.
	/// Unfortunately, we cannot use StatusCode directly, as it is not
	/// Serialize.
	Err((String, u16)),
}

impl TryFrom<&Result<CheckEmailOutput, TaskError>> for SingleShotReply {
	type Error = serde_json::Error;

	fn try_from(result: &Result<CheckEmailOutput, TaskError>) -> Result<Self, Self::Error> {
		match result {
			Ok(output) => Ok(Self::Ok(serde_json::to_vec(output)?)),
			Err(TaskError::Throttle(e)) => Ok(Self::Err((
				TaskError::Throttle(e.clone()).to_string(),
				StatusCode::TOO_MANY_REQUESTS.as_u16(),
			))),
			Err(e) => Ok(Self::Err((
				e.to_string(),
				StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
			))),
		}
	}
}

/// Send reply, in an "RPC mode", to the queue that initiated the request. We
/// follow this guide:
/// https://www.rabbitmq.com/tutorials/tutorial-six-javascript.html
///
/// This only applies for single-shot email verifications on the
/// /v1/check_email endpoint, and not to bulk verifications.
pub async fn send_single_shot_reply(
	channel: Arc<Channel>,
	delivery: &Delivery,
	worker_output: &Result<CheckEmailOutput, TaskError>,
) -> Result<(), anyhow::Error> {
	if let (Some(reply_to), Some(correlation_id)) = (
		delivery.properties.reply_to(),
		delivery.properties.correlation_id(),
	) {
		let properties = BasicProperties::default()
			.with_correlation_id(correlation_id.to_owned())
			.with_content_type("application/json".into());

		let single_shot_response = SingleShotReply::try_from(worker_output)?;
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
		bail!("Missing reply_to or correlation_id");
	}

	Ok(())
}
