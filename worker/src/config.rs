use config::Config;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
	/// Name of the worker.
	pub name: String,
	pub rabbitmq: RabbitMQConfig,
	pub throttle: ThrottleConfig,
	pub db: DBConfig,
	pub webhook: WebhookConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQConfig {
	pub url: String,
	/// Queue name to consume messages from.
	///
	/// The queue name MUST be one of the following:
	/// - "check.Smtp.gmail": verifies any Gmail email, using SMTP.
	/// - "check.Smtp.yahoo": verifies any Yahoo email, using SMTP.
	/// - "check.Smtp.hotmail.b2b": verifies any B2B Hotmail email, using SMTP.
	/// - "check.Smtp.hotmail.b2c": verifies any B2C Hotmail email (@outlook, @live, @hotmail), using SMTP.
	/// - "check.Smtp.hotmail.*": verifies any Hotmail email, using SMTP.
	///	- "check.Smtp.*": verifies any email, using SMTP.
	/// - "check.Headless.yahoo": verifies any Yahoo email, using a headless browser.
	/// - "check.Headless.hotmail.b2c": verifies any B2C Hotmail email (@outlook, @live, @hotmail) email, using a headless browser.
	/// - "check.Headless.*": verifies any email, using a headless browser.
	/// - "check.*": verifies any email, using the default method.
	pub queue: Queue,
	pub concurrency: u16,
}

/// Queue names that the worker can consume from.
#[derive(Debug, Clone)]
pub enum Queue {
	/// Verifies any Gmail email, using SMTP.
	SmtpGmail,
	/// Verifies any Yahoo email, using SMTP.
	SmtpYahoo,
	/// Verifies any B2B Hotmail email, using SMTP.
	SmtpHotmailB2B,
	/// Verifies any B2C Hotmail email (@outlook, @live, @hotmail), using SMTP.
	SmtpHotmailB2C,
	/// Verifies any Hotmail email, using SMTP.
	SmtpHotmailAll,
	/// Verifies any email, using SMTP.
	SmtpAll,
	/// Verifies any Yahoo email, using a headless browser.
	HeadlessYahoo,
	/// Verifies any B2C Hotmail email (@outlook, @live, @hotmail) email, using a headless browser.
	HeadlessHotmailB2C,
	/// Verifies any email, using a headless browser.
	HeadlessAll,
	/// Verifies any email, using the default method.
	All,
}

impl fmt::Display for Queue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Queue::SmtpGmail => write!(f, "check.Smtp.gmail"),
			Queue::SmtpYahoo => write!(f, "check.Smtp.yahoo"),
			Queue::SmtpHotmailB2B => write!(f, "check.Smtp.hotmail.b2b"),
			Queue::SmtpHotmailB2C => write!(f, "check.Smtp.hotmail.b2c"),
			Queue::SmtpHotmailAll => write!(f, "check.Smtp.hotmail.*"),
			Queue::SmtpAll => write!(f, "check.Smtp.*"),
			Queue::HeadlessYahoo => write!(f, "check.Headless.yahoo"),
			Queue::HeadlessHotmailB2C => write!(f, "check.Headless.hotmail.b2c"),
			Queue::HeadlessAll => write!(f, "check.Headless.*"),
			Queue::All => write!(f, "check.*"),
		}
	}
}

// Implement Deserialize for the Queue enum
impl<'de> Deserialize<'de> for Queue {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct QueueVisitor;

		impl<'de> Visitor<'de> for QueueVisitor {
			type Value = Queue;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a valid queue string")
			}

			fn visit_str<E>(self, value: &str) -> Result<Queue, E>
			where
				E: de::Error,
			{
				match value {
					"check.Smtp.gmail" => Ok(Queue::SmtpGmail),
					"check.Smtp.yahoo" => Ok(Queue::SmtpYahoo),
					"check.Smtp.hotmail.b2b" => Ok(Queue::SmtpHotmailB2B),
					"check.Smtp.hotmail.b2c" => Ok(Queue::SmtpHotmailB2C),
					"check.Smtp.hotmail.*" => Ok(Queue::SmtpHotmailAll),
					"check.Smtp.*" => Ok(Queue::SmtpAll),
					"check.Headless.yahoo" => Ok(Queue::HeadlessYahoo),
					"check.Headless.hotmail.b2c" => Ok(Queue::HeadlessHotmailB2C),
					"check.Headless.*" => Ok(Queue::HeadlessAll),
					"check.*" => Ok(Queue::All),
					_ => Err(de::Error::unknown_variant(
						value,
						&[
							"check.Smtp.gmail",
							"check.Smtp.yahoo",
							"check.Smtp.hotmail.b2b",
							"check.Smtp.hotmail.b2c",
							"check.Smtp.hotmail.*",
							"check.Smtp.*",
							"check.Headless.yahoo",
							"check.Headless.hotmail.b2c",
							"check.Headless.*",
							"check.*",
						],
					)),
				}
			}
		}

		deserializer.deserialize_str(QueueVisitor)
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThrottleConfig {
	pub max_requests_per_second: Option<u32>,
	pub max_requests_per_minute: Option<u32>,
	pub max_requests_per_hour: Option<u32>,
	pub max_requests_per_day: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DBConfig {
	pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
	pub url: Option<String>,
}

/// Load the worker configuration from the worker_config.toml file and from the
/// environment.
pub fn load_config() -> Result<WorkerConfig, config::ConfigError> {
	let cfg = Config::builder()
		.add_source(config::File::with_name("worker_config"))
		.add_source(config::Environment::with_prefix("RCH").separator("_"))
		.build()?;

	let cfg = cfg.try_deserialize::<WorkerConfig>()?;

	Ok(cfg)
}
