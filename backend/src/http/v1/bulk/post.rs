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

use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use lapin::Channel;
use lapin::{options::*, BasicProperties};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info};
use warp::http::StatusCode;
use warp::Filter;

use super::with_worker_db;
use crate::config::BackendConfig;
use crate::http::check_header;
use crate::http::v0::check_email::post::with_config;
use crate::http::CheckEmailRequest;
use crate::http::ReacherResponseError;
use crate::worker::consume::CHECK_EMAIL_QUEUE;
use crate::worker::do_work::CheckEmailJobId;
use crate::worker::do_work::CheckEmailTask;
use crate::worker::do_work::TaskWebhook;

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
	pg_pool: PgPool,
	body: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
	if body.input.is_empty() {
		return Err(ReacherResponseError::new(StatusCode::BAD_REQUEST, "Empty input").into());
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
	.map_err(ReacherResponseError::from)?;

	let n = body.input.len();
	let webhook = body.webhook.clone();
	let stream = futures::stream::iter(body.input.into_iter());

	let properties = BasicProperties::default()
		.with_content_type("application/json".into())
		.with_priority(1); // Low priority

	stream
		.map::<Result<_, ReacherResponseError>, _>(Ok)
		// Publish tasks to the queue, 10 at a time.
		.try_for_each_concurrent(10, |to_email| async {
			let input = CheckEmailRequest {
				to_email,
				..Default::default()
			}
			.to_check_email_input(Arc::clone(&config));

			let task = CheckEmailTask {
				input,
				job_id: CheckEmailJobId::Bulk(rec.id),
				webhook: webhook.clone(),
			};

			publish_task(
				config
					.must_worker_config()
					.map_err(ReacherResponseError::from)?
					.channel,
				task,
				properties.clone(),
			)
			.await
		})
		.await?;

	info!(
		target: LOG_TARGET,
		queue = CHECK_EMAIL_QUEUE,
		"Added {n} emails",
	);
	Ok(warp::reply::json(&Response { job_id: rec.id }))
}

/// Publish a task to the "check_email" queue.
pub async fn publish_task(
	channel: Arc<Channel>,
	task: CheckEmailTask,
	properties: BasicProperties,
) -> Result<(), ReacherResponseError> {
	let task_json = serde_json::to_vec(&task)?;
	channel
		.basic_publish(
			"",
			CHECK_EMAIL_QUEUE,
			BasicPublishOptions::default(),
			&task_json,
			properties,
		)
		.await
		.map_err(ReacherResponseError::from)?
		.await
		.map_err(ReacherResponseError::from)?;

	debug!(target: LOG_TARGET, email=?task.input.to_email, queue=?CHECK_EMAIL_QUEUE, "Published task");

	Ok(())
}

/// Create the `POST /bulk` endpoint.
/// The endpoint accepts list of email address and creates
/// a new job to check them.
pub fn v1_create_bulk_job(
	config: Arc<BackendConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v1" / "bulk")
		.and(warp::post())
		.and(check_header(Arc::clone(&config)))
		.and(with_config(Arc::clone(&config)))
		.and(with_worker_db(config))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and(warp::body::content_length_limit(1024 * 1024 * 50))
		.and(warp::body::json())
		.and_then(http_handler)
		// View access logs by setting `RUST_LOG=reacher_backend`.
		.with(warp::log(LOG_TARGET))
}
