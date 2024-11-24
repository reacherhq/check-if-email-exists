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

use super::check_email::{do_check_email_work, CheckEmailTask, TaskError};
use super::preprocess::{do_preprocess_work, PreprocessTask};
use super::response::send_single_shot_reply;
use crate::config::{BackendConfig, RabbitMQQueues, ThrottleConfig};
use anyhow::Context;
use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub const MAX_QUEUE_PRIORITY: u8 = 5;

/// Set up the RabbitMQ connection and declare all queues. This creates two
/// channels, one for checking emails and one for preprocessing. The
/// preprocessing channel is used to figure out the email provider and route
/// the message to the correct queue.
///
/// The check channel is used to consume messages from the queues. It has a
/// global prefetch limit set to the concurrency limit.
///
/// Returns a tuple of (check_channel, preprocess_channel).
pub async fn setup_rabbit_mq(
	config: Arc<BackendConfig>,
) -> Result<(Channel, Channel), anyhow::Error> {
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(config.backend_name.clone().into());

	let worker_config = config.must_worker_config();
	let conn = Connection::connect(&worker_config.rabbitmq.url, options)
		.await
		.with_context(|| format!("Connecting to rabbitmq {}", &worker_config.rabbitmq.url))?;
	let check_channel = conn.create_channel().await?;
	let preprocess_channel = conn.create_channel().await?;

	info!(target: LOG_TARGET, backend=?config.backend_name,state=?conn.status().state(), "Connected to AMQP broker");

	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), MAX_QUEUE_PRIORITY.into());

	// Assert all queues are declared.
	for queue in RabbitMQQueues::All.into_queues().iter() {
		check_channel
			.queue_declare(
				format!("{}", queue).as_str(),
				QueueDeclareOptions {
					durable: true,
					..Default::default()
				},
				queue_args.clone(),
			)
			.await?;
	}

	// Set up prefetch (concurrency) limit using qos
	check_channel
		.basic_qos(
			worker_config.rabbitmq.concurrency,
			// Set global to true to apply to all consumers.
			// ref: https://www.rabbitmq.com/docs/consumer-prefetch#independent-consumers
			BasicQosOptions { global: true },
		)
		.await?;

	preprocess_channel
		.queue_declare(
			"preprocess",
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			queue_args,
		)
		.await?;

	info!(target: LOG_TARGET, queues=?worker_config.rabbitmq.queues.into_queues(), concurrency=?worker_config.rabbitmq.concurrency, "Worker will start consuming messages");

	Ok((check_channel, preprocess_channel))
}

/// Start the worker to consume messages from the queue.
pub async fn run_worker(
	config: Arc<BackendConfig>,
	pg_pool: PgPool,
	check_channel: Arc<Channel>,
	preprocess_channel: Arc<Channel>,
) -> Result<(), anyhow::Error> {
	tokio::try_join!(
		consume_preprocess(
			Arc::clone(&config),
			Arc::clone(&check_channel),
			preprocess_channel
		),
		consume_check_email(Arc::clone(&config), pg_pool, check_channel)
	)?;

	Ok(())
}

/// Consume "Preprocess" queue, by figuring out the email provider and routing
/// (i.e. re-publishing) to the correct queue.
async fn consume_preprocess(
	config: Arc<BackendConfig>,
	check_channel: Arc<Channel>,
	preprocess_channel: Arc<Channel>,
) -> Result<(), anyhow::Error> {
	let mut consumer = preprocess_channel
		.basic_consume(
			"preprocess",
			format!("{}-preprocess", &config.backend_name).as_str(),
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await?;

	// Loop over the incoming messages
	while let Some(delivery) = consumer.next().await {
		let delivery = delivery?;
		let payload = serde_json::from_slice::<PreprocessTask>(&delivery.data)?;
		debug!(target: LOG_TARGET, email=payload.input.to_email, "New Preprocess job");

		let channel_clone = Arc::clone(&check_channel);
		let config_clone = Arc::clone(&config);

		tokio::spawn(async move {
			if let Err(e) =
				do_preprocess_work(&payload, delivery, channel_clone, config_clone).await
			{
				error!(target: LOG_TARGET, email=payload.input.to_email, error=?e, "Error preprocessing message");
			}
		});
	}
	Ok(())
}

async fn consume_check_email(
	config: Arc<BackendConfig>,
	pg_pool: PgPool,
	channel: Arc<Channel>,
) -> Result<(), anyhow::Error> {
	let worker_config = config.must_worker_config();

	let throttle = Arc::new(Mutex::new(Throttle::new()));

	for queue in &worker_config.rabbitmq.queues.into_queues() {
		let channel_clone = Arc::clone(&channel);
		let config_clone = Arc::clone(&config);
		let pg_pool_clone = pg_pool.clone();
		let throttle_clone = Arc::clone(&throttle);
		let queue_clone = queue.clone();

		tokio::spawn(async move {
			let worker_config = config_clone.must_worker_config();

			let mut consumer = channel_clone
				.basic_consume(
					queue_clone.to_string().as_str(),
					format!("{}-{}", &config_clone.backend_name, &queue_clone).as_str(),
					BasicConsumeOptions::default(),
					FieldTable::default(),
				)
				.await?;

			// Loop over the incoming messages
			while let Some(delivery) = consumer.next().await {
				let delivery = delivery?;
				let payload = serde_json::from_slice::<CheckEmailTask>(&delivery.data)?;
				debug!(target: LOG_TARGET, queue=?queue_clone.to_string(), email=?payload.input.to_email, "Consuming message");

				// Reset throttle counters if needed
				throttle_clone.lock().await.reset_if_needed();

				// Check if we should throttle before fetching the next message
				if let Some(wait_duration) = throttle_clone
					.lock()
					.await
					.should_throttle(&worker_config.throttle)
				{
					info!(target: LOG_TARGET, wait=?wait_duration, email=?payload.input.to_email, "Too many requests, throttling");

					// For single-shot tasks, we return an error early, so that the user knows they need to retry.
					if payload.is_single_shot() {
						debug!(target: LOG_TARGET, email=payload.input.to_email, job_id=?payload.job_id, queue=?queue_clone.to_string(), "Rejecting single-shot email because of throttling");
						delivery
							.reject(BasicRejectOptions { requeue: false })
							.await?;

						send_single_shot_reply(
							Arc::clone(&channel_clone),
							&delivery,
							&Err(TaskError::Throttle(wait_duration)),
						)
						.await?;
					} else {
						// Put back the message into the same queue, so that other
						// workers can pick it up.
						delivery
							.reject(BasicRejectOptions { requeue: true })
							.await?;
						debug!(target: LOG_TARGET, email=payload.input.to_email, job_id=?payload.job_id, queue=?queue_clone.to_string(), "Requeued message because of throttling");
					}

					continue;
				}

				let config_clone2 = Arc::clone(&config_clone);
				let channel_clone2 = Arc::clone(&channel_clone);
				let pg_pool_clone2 = pg_pool_clone.clone();

				info!(target: LOG_TARGET, email=payload.input.to_email, job_id=?payload.job_id, queue=?queue_clone.to_string(), "Starting task");
				tokio::spawn(async move {
					if let Err(e) = do_check_email_work(
						&payload,
						delivery,
						channel_clone2,
						pg_pool_clone2,
						config_clone2,
					)
					.await
					{
						error!(target: LOG_TARGET, email=payload.input.to_email, error=?e, "Error processing message");
					}
				});

				// Increment throttle counters once we spawn the task
				throttle_clone.lock().await.increment_counters();
			}

			Ok::<(), anyhow::Error>(())
		});
	}

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
