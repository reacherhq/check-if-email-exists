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

use std::sync::Arc;

use check_if_email_exists::CheckEmailInputBuilder;
use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use lapin::Channel;
use lapin::{options::*, BasicProperties};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info};
use warp::Filter;

use super::error::{handle_rejection, BulkError};
use crate::config::BackendConfig;
use crate::http::check_header;
use crate::http::with_config;
use crate::http::with_db;
use crate::worker::task::{TaskPayload, TaskWebhook};

/// POST v1/bulk endpoint request body.
#[derive(Debug, Deserialize)]
struct Request {
	input: Vec<String>,
	webhook: Option<TaskWebhook>,
}

/// POST v1/bulk endpoint response body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Response {
	job_id: i32,
}

async fn http_handler(
	config: Arc<BackendConfig>,
	channel: Arc<Channel>,
	pg_pool: PgPool,
	body: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
	if body.input.is_empty() {
		return Err(BulkError::EmptyInput.into());
	}

	// create job entry
	let rec = sqlx::query!(
		r#"
		INSERT INTO v1_bulk_job (total_records)
		VALUES ($1)
		RETURNING id
		"#,
		body.input.len() as i32
	)
	.fetch_one(&pg_pool)
	.await
	.map_err(|e| BulkError::from(e))?;

	let payloads = body.input.iter().map(|email| {
		let input = CheckEmailInputBuilder::default()
			.to_email(email.to_string())
			.from_email(config.from_email.clone())
			.hello_name(config.hello_name.clone())
			.proxy(config.proxy.clone())
			.build()
			.map_err(BulkError::from)?;

		Ok(TaskPayload {
			input,
			job_id: rec.id,
			webhook: body.webhook.clone(),
		})
	});
	let payloads = payloads.collect::<Result<Vec<_>, BulkError>>()?;

	let n = payloads.len();
	let stream = futures::stream::iter(payloads);

	stream
		.map::<Result<_, BulkError>, _>(Ok)
		.try_for_each_concurrent(10, |payload| {
			let channel = Arc::clone(&channel);
			let properties = BasicProperties::default()
				.with_content_type("application/json".into())
				.with_priority(1);

			async move {
				let payload_u8 = serde_json::to_vec(&payload)?;
				let queue_name = "preprocess";
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

	info!(
		target: LOG_TARGET,
		queue = "preprocess",
		"Added {n} emails to the queue",
	);
	Ok(warp::reply::json(&Response { job_id: rec.id }))
}

/// Create the `POST /bulk` endpoint.
/// The endpoint accepts list of email address and creates
/// a new job to check them.
pub fn create_bulk_job(
	config: Arc<BackendConfig>,
	channel: Arc<Channel>,
	pg_pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v1" / "bulk")
		.and(warp::post())
		.and(check_header(Arc::clone(&config)))
		.and(with_config(config))
		.and(with_channel(channel))
		.and(with_db(pg_pool))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and(warp::body::content_length_limit(1024 * 1024 * 50))
		.and(warp::body::json())
		.and_then(http_handler)
		.recover(handle_rejection)
		// View access logs by setting `RUST_LOG=reacher_backend`.
		.with(warp::log(LOG_TARGET))
}

/// Warp filter that extracts lapin Channel.
fn with_channel(
	channel: Arc<Channel>,
) -> impl Filter<Extract = (Arc<Channel>,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || Arc::clone(&channel))
}
