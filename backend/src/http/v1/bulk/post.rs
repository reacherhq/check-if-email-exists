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

//! This file implements the `POST /v1/bulk` endpoint.

use check_if_email_exists::CheckEmailInput;
use check_if_email_exists::CheckEmailInputProxy;
use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use lapin::Channel;
use lapin::{options::*, BasicProperties};
use serde::{Deserialize, Serialize};
use tracing::debug;
use warp::Filter;

use super::error::BulkError;
use crate::check::check_header;
use crate::worker::check_email::CheckEmailPayload;
use crate::worker::check_email::CheckEmailWebhook;

/// Endpoint request body.
#[derive(Debug, Deserialize)]
struct CreateBulkRequest {
	input: Vec<String>,
	proxy: Option<CheckEmailInputProxy>,
	hello_name: Option<String>,
	from_email: Option<String>,
	webhook: Option<CheckEmailWebhook>,
}

/// Endpoint response body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CreateBulkResponse {
	message: String,
}

async fn create_bulk_request(
	channel: Channel,
	body: CreateBulkRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
	if body.input.is_empty() {
		return Err(BulkError::EmptyInput.into());
	}

	let payloads = body.input.iter().map(|email| {
		let mut input = CheckEmailInput::new(email.to_string());
		if let Some(from_email) = &body.from_email {
			input.set_from_email(from_email.clone());
		}
		if let Some(hello_name) = &body.hello_name {
			input.set_hello_name(hello_name.clone());
		}
		if let Some(proxy) = &body.proxy {
			input.set_proxy(proxy.clone());
		}

		CheckEmailPayload {
			input,
			webhook: body.webhook.clone(),
		}
	});

	let n = payloads.len();
	let stream = futures::stream::iter(payloads);

	stream
		.map::<Result<_, BulkError>, _>(Ok)
		.try_for_each_concurrent(10, |payload| {
			let channel = channel.clone();
			let properties: lapin::protocol::basic::AMQPProperties = BasicProperties::default()
				.with_content_type("application/json".into())
				.with_priority(1);

			async move {
				let payload_u8 = serde_json::to_vec(&payload)?;
				let queue_name = "check_email.Smtp"; // TODO We might want to make this configurable.
				channel
					.basic_publish(
						"",
						queue_name,
						BasicPublishOptions::default(),
						&payload_u8,
						properties,
					)
					.await?
					.await?;

				debug!(target: LOG_TARGET, email=?payload.input.to_email, queue=?queue_name, "Enqueued");

				Ok(())
			}
		})
		.await?;

	Ok(warp::reply::json(&CreateBulkResponse {
		message: format!("Successfully added {n} emails to the queue"),
	}))
}

/// Create the `POST /bulk` endpoint.
/// The endpoint accepts list of email address and creates
/// a new job to check them.
pub fn create_bulk_job(
	o: Channel,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v1" / "bulk")
		.and(warp::post())
		.and(check_header())
		.and(with_channel(o))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(create_bulk_request)
		// View access logs by setting `RUST_LOG=reacher_backend`.
		.with(warp::log(LOG_TARGET))
}

/// Warp filter that extracts lapin Channel.
fn with_channel(
	channel: Channel,
) -> impl Filter<Extract = (Channel,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || channel.clone())
}
