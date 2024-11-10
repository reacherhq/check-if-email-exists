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
use check_if_email_exists::{CheckEmailInputProxy, SentryConfig};
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
	pub proxy: Option<CheckEmailInputProxy>,

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
	/// Queues names are in the format "check.<verif_method>.<provider>", where:
	/// - <verif_method> is the verification method to use. It can be "Smtp",
	/// "Headless", or "Api". Note that not all verification methods are
	/// available for all providers. For example, currently the Headless method
	/// is not available for Gmail. However, the Smtp method is available for
	/// all providers. The "*" wildcard can be used to match any verification
	/// method.
	/// - <provider> is the email provider to verify. It can be "gmail", "yahoo",
	/// "hotmail", or "*". The "*" provider is a wildcard that matches any
	/// provider.
	///
	/// The queue name MUST be one of the following:
	/// - "check.*.gmail": subcribe exclusively to Gmail emails.
	/// - "check.*.yahoo": subcribe exclusively to Yahoo emails, both Headless and SMTP.
	/// - "check.Smtp.yahoo": subcribe exclusively to Yahoo emails where the verification method is explicity set to SMTP.
	/// - "check.Headless.yahoo": subcribe exclusively to Yahoo emails where the verification method is unset, or set to Headless (default method).
	/// - "check.*.hotmail.*": subcribe exclusively to Hotmail emails, both B2B and B2C, both Headless and SMTP.
	/// - "check.*.hotmail.b2b": subcribe exclusively to B2B Hotmail emails, both Headless and SMTP.
	/// - "check.*.hotmail.b2c": subcribe exclusively to B2C Hotmail email (@outlook, @live, @hotmail), both Headless and SMTP.
	/// - "check.Smtp.hotmail.b2c": subcribe exclusively to B2C Hotmail email (@outlook, @live, @hotmail) whose verification method is explicity set to SMTP.
	/// - "check.Headless.hotmail.b2c": subcribe exclusively to B2C Hotmail email (@outlook, @live, @hotmail) whose verification method is unset, or set to Headless (default method).
	///	- "check.Smtp.#": subcribe any to email whose default verification method is SMTP.
	/// - "check.Headless.#": subcribe any to email whose default verification method is Headless.
	/// - "check.#": subcribe any to email.
	pub queue: Queue,
	pub concurrency: u16,
}

/// Queue names that the worker can consume from.
#[derive(Debug, Clone)]
pub enum Queue {
	AllGmail,
	AllYahoo,
	SmtpYahoo,
	HeadlessYahoo,
	AllHotmail,
	AllHotmailB2B,
	AllHotmailB2C,
	SmtpHotmailB2C,
	HeadlessHotmailB2C,
	SmtpAll,
	HeadlessAll,
	All,
}

impl fmt::Display for Queue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Queue::AllGmail => write!(f, "check.*.gmail"),
			Queue::AllYahoo => write!(f, "check.*.yahoo"),
			Queue::SmtpYahoo => write!(f, "check.Smtp.yahoo"),
			Queue::HeadlessYahoo => write!(f, "check.Headless.yahoo"),
			Queue::AllHotmail => write!(f, "check.*.hotmail.*"),
			Queue::AllHotmailB2B => write!(f, "check.*.hotmail.b2b"),
			Queue::AllHotmailB2C => write!(f, "check.*.hotmail.b2c"),
			Queue::SmtpHotmailB2C => write!(f, "check.Smtp.hotmail.b2c"),
			Queue::HeadlessHotmailB2C => write!(f, "check.Headless.hotmail.b2c"),
			Queue::SmtpAll => write!(f, "check.Smtp.#"),
			Queue::HeadlessAll => write!(f, "check.Headless.#"),
			Queue::All => write!(f, "check.#"),
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
					"check.*.gmail" => Ok(Queue::AllGmail),
					"check.*.yahoo" => Ok(Queue::AllYahoo),
					"check.Smtp.yahoo" => Ok(Queue::SmtpYahoo),
					"check.Headless.yahoo" => Ok(Queue::HeadlessYahoo),
					"check.*.hotmail.*" => Ok(Queue::AllHotmail),
					"check.*.hotmail.b2b" => Ok(Queue::AllHotmailB2B),
					"check.*.hotmail.b2c" => Ok(Queue::AllHotmailB2C),
					"check.Smtp.hotmail.b2c" => Ok(Queue::SmtpHotmailB2C),
					"check.Headless.hotmail.b2c" => Ok(Queue::HeadlessHotmailB2C),
					"check.Smtp.#" => Ok(Queue::SmtpAll),
					"check.Headless.#" => Ok(Queue::HeadlessAll),
					"check.#" => Ok(Queue::All),
					_ => Err(de::Error::unknown_variant(
						value,
						&[
							"check.*.gmail",
							"check.*.yahoo",
							"check.Smtp.yahoo",
							"check.Headless.yahoo",
							"check.*.hotmail.*",
							"check.*.hotmail.b2b",
							"check.*.hotmail.b2c",
							"check.Smtp.hotmail.b2c",
							"check.Headless.hotmail.b2c",
							"check.Smtp.#",
							"check.Headless.#",
							"check.#",
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
