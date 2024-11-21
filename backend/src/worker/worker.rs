use super::task::{process_queue_message, TaskPayload};
use crate::config::{BackendConfig, Queue, ThrottleConfig};
use crate::worker::task::preprocess;
use check_if_email_exists::LOG_TARGET;
use futures::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;
use tracing::{debug, error, info};

pub async fn setup_rabbit_mq(config: Arc<BackendConfig>) -> Result<Channel, lapin::Error> {
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(config.backend_name.clone().into());

	let worker_config = config.must_worker_config();
	let conn = Connection::connect(&worker_config.rabbitmq.url, options).await?;
	let channel = conn.create_channel().await?;

	info!(target: LOG_TARGET, backend=?config.backend_name,state=?conn.status().state(), "Connected to AMQP broker");

	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), 5.into());

	// Assert all queues are declared.
	let queues = vec![
		Queue::GmailSmtp,
		Queue::HotmailB2BSmtp,
		Queue::HotmailB2CSmtp,
		Queue::HotmailB2CHeadless,
		Queue::YahooSmtp,
		Queue::YahooHeadless,
		Queue::EverythingElseSmtp,
	];
	for queue in queues.iter() {
		channel
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
	channel
		.queue_declare(
			"preprocess",
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
			worker_config.rabbitmq.concurrency,
			// Set global to true to apply to all consumers.
			// ref: https://www.rabbitmq.com/docs/consumer-prefetch#independent-consumers
			BasicQosOptions { global: true },
		)
		.await?;

	info!(target: LOG_TARGET, queues=?worker_config.rabbitmq.queues, concurrency=?worker_config.rabbitmq.concurrency, "Worker will start consuming messages");

	Ok(channel)
}

/// Start the worker to consume messages from the queue.
pub async fn run_worker(
	config: Arc<BackendConfig>,
	pg_pool: PgPool,
	channel: Arc<Channel>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	tokio::try_join!(
		consume_preprocess(Arc::clone(&config), Arc::clone(&channel)),
		consume_check_email(Arc::clone(&config), pg_pool, Arc::clone(&channel))
	)?;

	Ok(())
}

/// Consume "Preprocess" queue, by figuring out the email provider and routing
/// (i.e. re-publishing) to the correct queue.
async fn consume_preprocess(
	config: Arc<BackendConfig>,
	channel: Arc<Channel>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let mut consumer = channel
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
		let payload = serde_json::from_slice::<TaskPayload>(&delivery.data)?;
		debug!(target: LOG_TARGET, email=payload.input.to_email, "New Preprocess job");

		let channel_clone = Arc::clone(&channel);

		tokio::spawn(async move {
			if let Err(e) = preprocess(&payload, delivery, channel_clone).await {
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let worker_config = config.must_worker_config();

	for queue in &worker_config.rabbitmq.queues {
		let channel = Arc::clone(&channel);
		let config = Arc::clone(&config);
		let pg_pool = pg_pool.clone();
		let queue = queue.clone();

		tokio::spawn(async move {
			let worker_config = config.must_worker_config();

			let mut consumer = channel
				.basic_consume(
					queue.to_string().as_str(),
					format!("{}-{}", &config.backend_name, &queue).as_str(),
					BasicConsumeOptions::default(),
					FieldTable::default(),
				)
				.await?;
			debug!(target: LOG_TARGET, queue=?queue, "Consuming messages");

			let mut throttle = Throttle::new();

			// Loop over the incoming messages
			while let Some(delivery) = consumer.next().await {
				let delivery = delivery?;
				let payload = serde_json::from_slice::<TaskPayload>(&delivery.data)?;
				info!(target: LOG_TARGET, email=payload.input.to_email, "New Check job");

				// Reset throttle counters if needed
				throttle.reset_if_needed();

				let config = Arc::clone(&config);
				let channel = Arc::clone(&channel);
				let pg_pool = pg_pool.clone();

				tokio::spawn(async move {
					if let Err(e) =
						process_queue_message(&payload, delivery, channel, pg_pool, config).await
					{
						error!(target: LOG_TARGET, email=payload.input.to_email, error=?e, "Error processing message");
					}
				});

				// Increment throttle counters once we spawn the task
				throttle.increment_counters();

				// Check if we should throttle before fetching the next message
				if let Some(wait_duration) = throttle.should_throttle(&worker_config.throttle) {
					info!(target: LOG_TARGET, wait=?wait_duration, "Too many requests, throttling");
					sleep(wait_duration).await;
					continue;
				}
			}

			Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
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
