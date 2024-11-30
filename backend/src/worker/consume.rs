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
use super::response::send_single_shot_reply;
use crate::config::{BackendConfig, RabbitMQConfig, ThrottleConfig};
use crate::worker::do_work::CheckEmailJobId;
use anyhow::Context;
use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

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

	let throttle = Arc::new(Mutex::new(Throttle::new()));

	tokio::spawn(async move {
		let worker_config = config_clone.must_worker_config()?;

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

			// Reset throttle counters if needed
			throttle.lock().await.reset_if_needed();

			// Check if we should throttle before fetching the next message
			if let Some(wait_duration) = throttle
				.lock()
				.await
				.should_throttle(&worker_config.throttle)
			{
				info!(target: LOG_TARGET, wait=?wait_duration, email=?payload.input.to_email, "Too many requests, throttling");

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
							&Err(TaskError::Throttle(wait_duration)),
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

			info!(target: LOG_TARGET, email=payload.input.to_email, job_id=?payload.job_id, "Starting task");
			tokio::spawn(async move {
				if let Err(e) =
					do_check_email_work(&payload, delivery, channel_clone2, config_clone2).await
				{
					error!(target: LOG_TARGET, email=payload.input.to_email, error=?e, "Error processing message");
				}
			});

			// Increment throttle counters once we spawn the task
			throttle.lock().await.increment_counters();
		}

		Ok::<(), anyhow::Error>(())
	});

	Ok(())
}

#[derive(Clone)]
struct Throttle {
	requests_per_second: u32,
	requests_per_minute: u32,
	requests_per_hour: u32,
	requests_per_day: u32,
	last_reset_second: Instant,
	last_reset_minute: Instant,
	last_reset_hour: Instant,
	last_reset_day: Instant,
}

impl Throttle {
	fn new() -> Self {
		let now = Instant::now();
		Throttle {
			requests_per_second: 0,
			requests_per_minute: 0,
			requests_per_hour: 0,
			requests_per_day: 0,
			last_reset_second: now,
			last_reset_minute: now,
			last_reset_hour: now,
			last_reset_day: now,
		}
	}

	fn reset_if_needed(&mut self) {
		let now = Instant::now();

		// Reset per-second counter
		if now.duration_since(self.last_reset_second) >= Duration::from_secs(1) {
			self.requests_per_second = 0;
			self.last_reset_second = now;
		}

		// Reset per-minute counter
		if now.duration_since(self.last_reset_minute) >= Duration::from_secs(60) {
			self.requests_per_minute = 0;
			self.last_reset_minute = now;
		}

		// Reset per-hour counter
		if now.duration_since(self.last_reset_hour) >= Duration::from_secs(3600) {
			self.requests_per_hour = 0;
			self.last_reset_hour = now;
		}

		// Reset per-day counter
		if now.duration_since(self.last_reset_day) >= Duration::from_secs(86400) {
			self.requests_per_day = 0;
			self.last_reset_day = now;
		}
	}

	fn increment_counters(&mut self) {
		self.requests_per_second += 1;
		self.requests_per_minute += 1;
		self.requests_per_hour += 1;
		self.requests_per_day += 1;
	}

	fn should_throttle(&self, config: &ThrottleConfig) -> Option<Duration> {
		let now = Instant::now();

		if let Some(max_per_second) = config.max_requests_per_second {
			if self.requests_per_second >= max_per_second {
				return Some(Duration::from_secs(1) - now.duration_since(self.last_reset_second));
			}
		}

		if let Some(max_per_minute) = config.max_requests_per_minute {
			if self.requests_per_minute >= max_per_minute {
				return Some(Duration::from_secs(60) - now.duration_since(self.last_reset_minute));
			}
		}

		if let Some(max_per_hour) = config.max_requests_per_hour {
			if self.requests_per_hour >= max_per_hour {
				return Some(Duration::from_secs(3600) - now.duration_since(self.last_reset_hour));
			}
		}

		if let Some(max_per_day) = config.max_requests_per_day {
			if self.requests_per_day >= max_per_day {
				return Some(Duration::from_secs(86400) - now.duration_since(self.last_reset_day));
			}
		}

		None
	}
}
