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

use check_if_email_exists::config::ReacherConfig;
use check_if_email_exists::SentryConfig;
use config::Config;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
	/// Name of the backend.
	pub backend_name: String,

	/** Reacher config*/
	pub from_email: String,
	pub hello_name: String,
	pub webdriver_addr: String,

	/** Backend-specific config*/
	/// Backend host
	pub http_host: String,
	/// Backend port
	pub http_port: u16,
	/// Shared secret between a trusted client and the backend.
	pub header_secret: Option<String>,

	/// Worker configuration, only present if the backend is a worker.
	pub worker: WorkerConfig,

	/// Sentry configuration to report errors.
	pub sentry: Option<SentryConfig>,
}

impl BackendConfig {
	pub fn get_reacher_config(&self) -> ReacherConfig {
		ReacherConfig {
			backend_name: self.backend_name.clone(),
			sentry: self.sentry.clone(),
			webdriver_addr: self.webdriver_addr.clone(),
		}
	}

	/// Get the worker configuration.
	///
	/// # Panics
	///
	/// Panics if the worker configuration is missing.
	pub fn must_worker_config(&self) -> MustWorkerConfig {
		MustWorkerConfig {
			throttle: self
				.worker
				.throttle
				.clone()
				.expect("worker.throttle is missing"),
			rabbitmq: self
				.worker
				.rabbitmq
				.clone()
				.expect("worker.rabbitmq is missing"),
			webhook: self.worker.webhook.clone(),
			postgres: self
				.worker
				.postgres
				.clone()
				.expect("worker.postgres is missing"),
		}
	}
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct WorkerConfig {
	pub enable: bool,

	/// Throttle configuration for the worker.
	pub throttle: Option<ThrottleConfig>,
	pub rabbitmq: Option<RabbitMQConfig>,
	/// Optional webhook configuration to send email verification results.
	pub webhook: Option<WebhookConfig>,
	/// Postgres database configuration to store email verification
	/// results.
	pub postgres: Option<PostgresConfig>,
}

/// Worker configuration that must be present if worker.enable is true.
#[derive(Debug, Deserialize, Clone)]
pub struct MustWorkerConfig {
	pub throttle: ThrottleConfig,
	pub rabbitmq: RabbitMQConfig,
	pub webhook: Option<WebhookConfig>,
	pub postgres: PostgresConfig,
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
pub struct PostgresConfig {
	pub db_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThrottleConfig {
	pub max_requests_per_second: Option<u32>,
	pub max_requests_per_minute: Option<u32>,
	pub max_requests_per_hour: Option<u32>,
	pub max_requests_per_day: Option<u32>,
}

impl ThrottleConfig {
	/// Create a new ThrottleConfig with no throttling.
	pub fn new_without_throttle() -> Self {
		Self {
			max_requests_per_second: None,
			max_requests_per_minute: None,
			max_requests_per_hour: None,
			max_requests_per_day: None,
		}
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
	pub url: String,
}

/// Load the worker configuration from the worker_config.toml file and from the
/// environment.
pub fn load_config() -> Result<BackendConfig, config::ConfigError> {
	let cfg = Config::builder()
		.add_source(config::File::with_name("backend_config"))
		.add_source(config::Environment::with_prefix("RCH"))
		.build()?;

	let cfg = cfg.try_deserialize::<BackendConfig>()?;

	Ok(cfg)
}
