use crate::check_email::process_queue_message;
use crate::check_email::WorkerPayload;
use crate::config::WorkerConfig;
use check_if_email_exists::LOG_TARGET;
use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;
use tracing::{error, info};

pub async fn setup_rabbit_mq(config: &WorkerConfig) -> Result<Channel, lapin::Error> {
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(config.name.clone().into());

	let conn = Connection::connect(&config.rabbitmq.url, options).await?;
	let channel = conn.create_channel().await?;

	info!(target: LOG_TARGET, backend=?config.name,state=?conn.status().state(), "Connected to AMQP broker");

	let queue_name = config.rabbitmq.queue.to_string();
	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), 5.into());

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

	// Set up prefetch (concurrency) limit using qos
	channel
		.basic_qos(
			config.rabbitmq.concurrency.into(),
			BasicQosOptions::default(),
		)
		.await?;

	info!(target: LOG_TARGET, queue=?queue_name, "Worker will start consuming messages");

	Ok(channel)
}

pub async fn run_worker(
	config: WorkerConfig,
	pg_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let channel = Arc::new(setup_rabbit_mq(&config).await?);

	let mut consumer = channel
		.basic_consume(
			&config.rabbitmq.queue.to_string(),
			&config.name,
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await?;

	let mut throttle = Throttle::new();

	// Loop over the incoming messages
	while let Some(delivery) = consumer.next().await {
		let delivery = delivery?;
		let payload = serde_json::from_slice::<WorkerPayload>(&delivery.data)?;
		info!(target: LOG_TARGET, email=payload.input.to_email, "New job");

		// Reset throttle counters if needed
		throttle.reset_if_needed();

		let config_clone = config.clone();
		let channel_clone = channel.clone();
		let pg_pool_clone = pg_pool.clone();

		tokio::spawn(async move {
			if let Err(e) = process_queue_message(
				&payload,
				delivery,
				channel_clone,
				pg_pool_clone,
				config_clone,
			)
			.await
			{
				error!(target: LOG_TARGET, email=payload.input.to_email, error=?e, "Error processing message");
			}
		});

		// Increment throttle counters once we spawn the task
		throttle.increment_counters();

		// Check if we should throttle before fetching the next message
		if let Some(wait_duration) = throttle.should_throttle(&config) {
			info!(target: LOG_TARGET, wait=?wait_duration, "Too many requests, throttling");
			sleep(wait_duration).await;
			continue;
		}
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
