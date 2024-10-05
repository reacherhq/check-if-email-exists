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

use crate::check_email::process_queue_message;
use crate::config::WorkerConfig;
use check_if_email_exists::LOG_TARGET;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{error, info};

pub async fn setup_rabbit_mq(config: &WorkerConfig) -> Result<Channel, lapin::Error> {
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		// At the moment the reactor is only available for unix (ref: https://github.com/amqp-rs/reactor-trait/issues/1)
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(config.name.clone().into());

	let conn = Connection::connect(&config.rabbitmq.url, options).await?;

	let channel = conn.create_channel().await?;
	info!(target: LOG_TARGET, backend=?config.name,state=?conn.status().state(), "Connected to AMQP broker");

	let queue_name = config.rabbitmq.queue.to_string();
	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), 5.into()); // https://www.rabbitmq.com/priority.html

	channel
		.queue_declare(
			&queue_name,
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			queue_args,
		)
		.await?;

	info!(target: LOG_TARGET, queue=?queue_name, "Worker will start consuming messages");

	Ok(channel)
}

#[derive(Clone)]
pub struct Worker {
	config: WorkerConfig,
	channel: Arc<Channel>,
	semaphore: Arc<Semaphore>,
	throttle: Throttle,
	pg_pool: PgPool,
}

impl Worker {
	pub fn new(config: WorkerConfig, channel: Channel, pg_pool: PgPool) -> Self {
		Worker {
			config,
			channel: Arc::new(channel),
			semaphore: Arc::new(Semaphore::new(0)),
			throttle: Throttle::new(),
			pg_pool,
		}
	}

	pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let concurrency_limit: usize = self.config.rabbitmq.concurrency.into();
		self.semaphore = Arc::new(Semaphore::new(concurrency_limit));

		loop {
			let permit = self.semaphore.clone().acquire_owned().await?;

			// Reset throttle counters as needed
			self.throttle.reset_if_needed();

			// Check if throttling is needed
			if let Some(sleep_duration) = self.throttle.should_throttle(&self.config) {
				sleep(sleep_duration).await;
				continue;
			}

			// Fetch the message
			let thread_id = std::thread::current().id();
			match self.fetch_message().await {
				Ok(Some(delivery)) => {
					let config = self.config.clone(); // Clone config as it's shared across tasks
					let pg_pool = self.pg_pool.clone(); // Clone the database pool for task use
					let channel = Arc::clone(&self.channel); // Share channel across tasks

					tokio::spawn(async move {
						if let Err(e) =
							process_queue_message(delivery, channel, pg_pool, config).await
						{
							error!(target: LOG_TARGET, error=?e, thread_id=?thread_id, "Failed to process queue message");
						}
						drop(permit); // Release the permit when the job is done
					});

					// Increment throttle counters after fetching a message
					self.throttle.increment_counters();
				}
				Ok(None) => {
					drop(permit); // Release permit even if no message was fetched
				}
				Err(e) => {
					error!(target: LOG_TARGET, error=?e, thread_id=?thread_id, "Failed to fetch message");
					drop(permit); // Release permit on error
				}
			}
		}
	}

	async fn fetch_message(&self) -> Result<Option<lapin::message::BasicGetMessage>, lapin::Error> {
		let get_result = self
			.channel
			.basic_get(
				&self.config.rabbitmq.queue.to_string(),
				BasicGetOptions::default(),
			)
			.await;
		match get_result {
			Ok(Some(delivery)) => Ok(Some(delivery)),
			Ok(None) => Ok(None), // No messages
			e => e,               // Handle errors (e.g., log and retry)
		}
	}
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

	fn should_throttle(&self, config: &WorkerConfig) -> Option<Duration> {
		let now = Instant::now();

		if let Some(max_per_second) = config.throttle.max_requests_per_second {
			if self.requests_per_second >= max_per_second {
				return Some(Duration::from_secs(1) - now.duration_since(self.last_reset_second));
			}
		}

		if let Some(max_per_minute) = config.throttle.max_requests_per_minute {
			if self.requests_per_minute >= max_per_minute {
				return Some(Duration::from_secs(60) - now.duration_since(self.last_reset_minute));
			}
		}

		if let Some(max_per_hour) = config.throttle.max_requests_per_hour {
			if self.requests_per_hour >= max_per_hour {
				return Some(Duration::from_secs(3600) - now.duration_since(self.last_reset_hour));
			}
		}

		if let Some(max_per_day) = config.throttle.max_requests_per_day {
			if self.requests_per_day >= max_per_day {
				return Some(Duration::from_secs(86400) - now.duration_since(self.last_reset_day));
			}
		}

		None
	}
}
