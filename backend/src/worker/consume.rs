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

use super::do_work::{do_check_email_work, CheckEmailTask, TaskError};
use super::single_shot::send_single_shot_reply;
use crate::config::{BackendConfig, RabbitMQConfig};
use crate::worker::do_work::CheckEmailJobId;
use anyhow::Context;
use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use sentry_anyhow::capture_anyhow;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

/// Our RabbitMQ only has one queue: "check_email".
pub const CHECK_EMAIL_QUEUE: &str = "check_email";
pub const MAX_QUEUE_PRIORITY: u8 = 5;

/// Set up the RabbitMQ connection and declare the "check_email" queue.
///
/// The check channel is used to consume messages from the queue. It has a
/// global prefetch limit set to the concurrency limit.
///
/// Returns a the lapin Channel.
pub async fn setup_rabbit_mq(
	backend_name: &str,
	config: &RabbitMQConfig,
) -> Result<Channel, anyhow::Error> {
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(backend_name.into());

	let conn = Connection::connect(&config.url, options)
		.await
		.with_context(|| format!("Connecting to rabbitmq {}", &config.url))?;
	let channel = conn.create_channel().await?;

	info!(target: LOG_TARGET, backend=?backend_name,state=?conn.status().state(), "Connected to AMQP broker");

	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), MAX_QUEUE_PRIORITY.into());

	// Assert all queue is declared.
	channel
		.queue_declare(
			CHECK_EMAIL_QUEUE,
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			queue_args.clone(),
		)
		.await?;

	// Set up prefetch (concurrency) limit using qos
	channel
		.basic_qos(
			config.concurrency,
			// Set global to true to apply to all consumers, even though in our
			// case there's only one consumer.
			// ref: https://www.rabbitmq.com/docs/consumer-prefetch#independent-consumers
			BasicQosOptions { global: true },
		)
		.await?;

	info!(target: LOG_TARGET, queues=?CHECK_EMAIL_QUEUE, concurrency=?config.concurrency, "Worker will start consuming messages");

	Ok(channel)
}

/// Start the worker to consume messages from the queue.
pub async fn run_worker(config: Arc<BackendConfig>) -> Result<(), anyhow::Error> {
	consume_check_email(config).await
}

/// Consume "check_email" queue.
async fn consume_check_email(config: Arc<BackendConfig>) -> Result<(), anyhow::Error> {
	let config_clone = Arc::clone(&config);
	let worker_config = config_clone.must_worker_config()?;
	let channel = worker_config.channel;
	let throttle = config.get_throttle_manager();

	tokio::spawn(async move {
		let mut consumer = channel
			.basic_consume(
				CHECK_EMAIL_QUEUE,
				format!("{}-{}", &config_clone.backend_name, CHECK_EMAIL_QUEUE).as_str(),
				BasicConsumeOptions::default(),
				FieldTable::default(),
			)
			.await?;

		// Loop over the incoming messages
		while let Some(delivery) = consumer.next().await {
			let delivery = delivery?;
			let payload = serde_json::from_slice::<CheckEmailTask>(&delivery.data)?;
			debug!(target: LOG_TARGET, email=?payload.input.to_email, "Consuming message");

			// Check if we should throttle before fetching the next message
			if let Some(throttle_result) = throttle.check_throttle().await {
				// This line below will log every time the worker fetches from
				// RabbitMQ. It's noisy
				trace!(target: LOG_TARGET, wait=?throttle_result.delay, email=?payload.input.to_email, "Too many requests {}, throttling", throttle_result.limit_type);

				// For single-shot tasks, we return an error early, so that the user knows they need to retry.
				match payload.job_id {
					CheckEmailJobId::SingleShot => {
						debug!(target: LOG_TARGET, email=payload.input.to_email, job_id=?payload.job_id, "Rejecting single-shot email because of throttling");
						delivery
							.reject(BasicRejectOptions { requeue: false })
							.await?;

						send_single_shot_reply(
							Arc::clone(&channel),
							&delivery,
							&Err(TaskError::Throttle(throttle_result)),
						)
						.await?;
					}
					CheckEmailJobId::Bulk(_) => {
						// Put back the message into the same queue, so that other
						// workers can pick it up.
						delivery
							.reject(BasicRejectOptions { requeue: true })
							.await?;
						debug!(target: LOG_TARGET, email=payload.input.to_email, job_id=?payload.job_id, "Requeued message because of throttling");
					}
				}

				continue;
			}

			let config_clone2 = Arc::clone(&config_clone);
			let channel_clone2 = Arc::clone(&channel);

			info!(
				target: LOG_TARGET,
				email=payload.input.to_email,
				job_id=?payload.job_id,
				"Starting task"
			);
			tokio::spawn(async move {
				if let Err(e) =
					do_check_email_work(&payload, delivery, channel_clone2, config_clone2).await
				{
					error!(
						target: LOG_TARGET,
						email=payload.input.to_email,
						error=?e,
						"Error processing message"
					);
					capture_anyhow(&e);
				}
			});

			// Increment throttle counters once we spawn the task
			throttle.increment_counters().await;
		}

		Ok::<(), anyhow::Error>(())
	});

	Ok(())
}
