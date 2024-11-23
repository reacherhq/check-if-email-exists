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

#[cfg(feature = "worker")]
use crate::worker::task::TaskWebhook;
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
			#[cfg(feature = "worker")]
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
	#[cfg(feature = "worker")]
	pub webhook: Option<TaskWebhook>,
	/// Postgres database configuration to store email verification
	/// results.
	pub postgres: Option<PostgresConfig>,
}

/// Worker configuration that must be present if worker.enable is true.
#[derive(Debug, Deserialize, Clone)]
pub struct MustWorkerConfig {
	pub throttle: ThrottleConfig,
	pub rabbitmq: RabbitMQConfig,
	#[cfg(feature = "worker")]
	pub webhook: Option<TaskWebhook>,
	pub postgres: PostgresConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQConfig {
	pub url: String,
	/// Queues to consume emails from.
	///
	/// Queue names are in the format "check.<provider>.<verif_method>", where:
	/// - <verif_method> is the verification method to use. It can be "smtp",
	/// "headless", or "api". Note that _not_ all verification methods are
	/// available for all providers. For example, currently the Headless method
	/// is not available for Gmail. However, the Smtp method is available for
	/// all providers.
	/// - <provider> is the email provider to verify. It can be "gmail", "yahoo",
	/// "hotmail", or "everything_else".
	///
	/// Below is the exhaustive list of queue names that the worker can consume from:
	/// - "check.gmail.smtp": subcribe exclusively to Gmail emails.
	/// - "check.hotmail.b2b.smtp": subcribe exclusively to Hotmail B2B emails.
	/// - "check.hotmail.b2c.smtp": subcribe exclusively to Hotmail B2C emails where the verification method is explicity set to SMTP (default method is Headless).
	/// - "check.hotmail.b2c.headless": subcribe exclusively to Hotmail B2C emails where the verification method is explicity set to Headless (default method is Headless).
	/// - "check.yahoo.smtp": subcribe exclusively to Yahoo emails where the verification method is explicity set to SMTP (default method is Headless).
	/// - "check.yahoo.headless": subcribe exclusively to Yahoo emails where the verification method is explicity set to Headless (default method is Headless).
	/// - "check.everything_else.smtp": subcribe to all emails that are not Gmail, Yahoo, or Hotmail, the method used will be SMTP.
	pub queues: Vec<Queue>,
	/// Total number of concurrent messages that the worker can process, across
	/// all queues.
	pub concurrency: u16,
}

/// Queue names that the worker can consume from. Each email is routed to a
/// one and only one queue, based on the email provider and the verification
/// method. The worker can consume from multiple queues.
#[derive(Debug, Clone)]
pub enum Queue {
	GmailSmtp,
	HotmailB2BSmtp,
	HotmailB2CSmtp,
	HotmailB2CHeadless,
	YahooSmtp,
	YahooHeadless,
	EverythingElseSmtp,
}

impl fmt::Display for Queue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Queue::GmailSmtp => write!(f, "check.gmail.smtp"),
			Queue::HotmailB2BSmtp => write!(f, "check.hotmail.b2b.smtp"),
			Queue::HotmailB2CSmtp => write!(f, "check.hotmail.b2c.smtp"),
			Queue::HotmailB2CHeadless => write!(f, "check.hotmail.b2c.headless"),
			Queue::YahooSmtp => write!(f, "check.yahoo.smtp"),
			Queue::YahooHeadless => write!(f, "check.yahoo.headless"),
			Queue::EverythingElseSmtp => write!(f, "check.everything_else.smtp"),
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
					"check.gmail.smtp" => Ok(Queue::GmailSmtp),
					"check.hotmail.b2b.smtp" => Ok(Queue::HotmailB2BSmtp),
					"check.hotmail.b2c.smtp" => Ok(Queue::HotmailB2CSmtp),
					"check.hotmail.b2c.headless" => Ok(Queue::HotmailB2CHeadless),
					"check.yahoo.smtp" => Ok(Queue::YahooSmtp),
					"check.yahoo.headless" => Ok(Queue::YahooHeadless),
					"check.everything_else.smtp" => Ok(Queue::EverythingElseSmtp),
					_ => Err(de::Error::unknown_variant(
						value,
						&[
							"check.gmail.smtp",
							"check.hotmail.b2b.smtp",
							"check.hotmail.b2c.smtp",
							"check.hotmail.b2c.headless",
							"check.yahoo.smtp",
							"check.yahoo.headless",
							"check.everything_else.smtp",
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

/// Load the worker configuration from the worker_config.toml file and from the
/// environment.
pub fn load_config() -> Result<BackendConfig, config::ConfigError> {
	let cfg = Config::builder()
		.add_source(config::File::with_name("backend_config"))
		.add_source(config::Environment::with_prefix("RCH").separator("__"))
		.build()?;

	let cfg = cfg.try_deserialize::<BackendConfig>()?;

	Ok(cfg)
}
