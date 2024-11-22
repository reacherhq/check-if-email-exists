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

//! This file implements the `POST /v1/check_email` endpoint.

use check_if_email_exists::LOG_TARGET;
use futures::StreamExt;
use lapin::options::{
	BasicAckOptions, BasicConsumeOptions, BasicRejectOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel};
use std::sync::Arc;
use warp::http::StatusCode;
use warp::{http, Filter};

use crate::config::BackendConfig;
use crate::http::v0::check_email::post::CheckEmailRequest;
use crate::http::v1::bulk::post::publish_task;
use crate::http::v1::bulk::post::with_channel;
use crate::http::{check_header, with_config, ReacherResponseError};
use crate::worker::task::{SingleShotReply, TaskPayload};
use crate::worker::worker::MAX_QUEUE_PRIORITY;

/// The main endpoint handler that implements the logic of this route.
async fn http_handler(
	config: Arc<BackendConfig>,
	channel: Arc<Channel>,
	body: CheckEmailRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
	// The to_email field must be present
	if body.to_email.is_empty() {
		return Err(ReacherResponseError::new(
			http::StatusCode::BAD_REQUEST,
			"to_email field is required.",
		)
		.into());
	}

	// Follow this RPC tutorial:
	// https://www.rabbitmq.com/tutorials/tutorial-six-javascript#callback-queue
	let correlation_id = uuid::Uuid::new_v4();
	let reply_queue = channel
		.queue_declare(
			"", // Let RabbitMQ generate a unique name
			QueueDeclareOptions {
				auto_delete: true,
				durable: false,
				exclusive: true,
				..Default::default()
			},
			FieldTable::default(),
		)
		.await
		.map_err(ReacherResponseError::from)?;

	let check_email_input = body.to_check_email_input(Arc::clone(&config));
	let properties = BasicProperties::default()
		.with_content_type("application/json".into())
		.with_priority(MAX_QUEUE_PRIORITY) // Highes priority
		.with_correlation_id(correlation_id.to_string().into())
		.with_reply_to(reply_queue.name().to_owned());

	publish_task(
		channel.clone(),
		TaskPayload {
			input: check_email_input,
			job_id: None,
			webhook: None,
		},
		properties,
	)
	.await?;

	// Wait for the callback from the worker.
	let mut consumer = channel
		.basic_consume(
			reply_queue.name().as_str(),
			format!("rpc.{}", correlation_id).as_str(),
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await
		.map_err(ReacherResponseError::from)?;

	while let Some(delivery) = consumer.next().await {
		let delivery = delivery.map_err(ReacherResponseError::from)?;

		if delivery
			.properties
			.correlation_id()
			.as_ref()
			.map(|s| s.as_str())
			== Some(correlation_id.to_string().as_str())
		{
			delivery
				.ack(BasicAckOptions::default())
				.await
				.map_err(ReacherResponseError::from)?;

			let single_shot_response = serde_json::from_slice::<SingleShotReply>(&delivery.data)
				.map_err(ReacherResponseError::from)?;
			let status_code = StatusCode::from_u16(single_shot_response.code)
				.map_err(ReacherResponseError::from)?;

			if !status_code.is_success() {
				return Err(ReacherResponseError::new(
					status_code,
					String::from_utf8_lossy(&single_shot_response.body).to_string(),
				)
				.into());
			}

			return Ok(warp::reply::with_header(
				single_shot_response.body,
				"Content-Type",
				"application/json",
			));
		} else {
			delivery
				.reject(BasicRejectOptions { requeue: false })
				.await
				.map_err(ReacherResponseError::from)?;
			return Err(ReacherResponseError::new(
				http::StatusCode::INTERNAL_SERVER_ERROR,
				"Failed to get a reply from the worker.",
			)
			.into());
		}
	}

	// Run the future to check an email.
	Err(ReacherResponseError::new(
		http::StatusCode::INTERNAL_SERVER_ERROR,
		"Failed to get a reply from the worker.",
	)
	.into())
}

/// Create the `POST /v1/check_email` endpoint.
pub fn v1_check_email<'a>(
	config: Arc<BackendConfig>,
	channel: Arc<Channel>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + 'a {
	warp::path!("v1" / "check_email")
		.and(warp::post())
		.and(check_header(Arc::clone(&config)))
		.and(with_config(config))
		.and(with_channel(channel))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(http_handler)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log(LOG_TARGET))
}
