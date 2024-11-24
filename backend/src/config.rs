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
use crate::worker::check_email::TaskWebhook;
use check_if_email_exists::config::ReacherConfig;
use check_if_email_exists::{
	CheckEmailInputProxy, GmailVerifMethod, HotmailB2BVerifMethod, HotmailB2CVerifMethod,
	SentryConfig, YahooVerifMethod,
};
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

	/// Verification method configuration.
	pub verif_method: VerifMethodConfig,

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

#[derive(Debug, Deserialize, Clone)]
pub struct VerifMethodConfig {
	/// Verification method for Gmail emails.
	pub gmail: GmailVerifMethod,
	/// Verification method for Hotmail B2B emails.
	pub hotmailb2b: HotmailB2BVerifMethod,
	/// Verification method for Hotmail B2C emails.
	pub hotmailb2c: HotmailB2CVerifMethod,
	/// Verification method for Yahoo emails.
	pub yahoo: YahooVerifMethod,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Clone)]
pub enum RabbitMQQueues {
	All,
	Only(Vec<Queue>),
}

/// Deserialize RabbitMQQueues from a string or a list of strings.
/// If the value is "all", then we return RabbitMQQueues::All.
/// If the value is a list of strings, then we return RabbitMQQueues::Only.
impl<'de> Deserialize<'de> for RabbitMQQueues {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct RabbitMQQueuesVisitor;

		impl<'de> Visitor<'de> for RabbitMQQueuesVisitor {
			type Value = RabbitMQQueues;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a string 'all' or a list of queue strings")
			}

			fn visit_str<E>(self, value: &str) -> Result<RabbitMQQueues, E>
			where
				E: de::Error,
			{
				if value == "all" {
					Ok(RabbitMQQueues::All)
				} else {
					Err(de::Error::unknown_variant(value, &["all"]))
				}
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<RabbitMQQueues, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut queues = Vec::new();
				while let Some(value) = seq.next_element()? {
					queues.push(value);
				}
				Ok(RabbitMQQueues::Only(queues))
			}
		}

		deserializer.deserialize_any(RabbitMQQueuesVisitor)
	}
}

impl RabbitMQQueues {
	pub fn into_queues(self) -> Vec<Queue> {
		match self {
			RabbitMQQueues::All => vec![
				Queue::Gmail,
				Queue::HotmailB2B,
				Queue::HotmailB2C,
				Queue::Yahoo,
				Queue::EverythingElse,
			],
			RabbitMQQueues::Only(queues) => queues.clone(),
		}
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQConfig {
	pub url: String,
	/// Queues to consume emails from. By default the worker consumes from all
	/// queues.
	///
	/// If you wish to consume from only a subset of queues, you can uncomment
	/// the line `queues = "all"`, and then specify the queues you want to
	/// consume from.
	///
	/// Below is the exhaustive list of queue names that the worker can consume from:
	/// - "check.gmail": subcribe exclusively to Gmail emails.
	/// - "check.hotmailb2b": subcribe exclusively to Hotmail B2B emails.
	/// - "check.hotmailb2c": subcribe exclusively to Hotmail B2C emails.
	/// - "check.yahoo": subcribe exclusively to Yahoo emails.
	/// - "check.everything_else": subcribe to all emails that are not Gmail, Yahoo, or Hotmail.
	///
	/// queues = ["check.gmail", "check.hotmailb2b", "check.hotmailb2c", "check.yahoo", "check.everything_else"]
	pub queues: RabbitMQQueues,
	/// Total number of concurrent messages that the worker can process, across
	/// all queues.
	pub concurrency: u16,
}

/// Queue names that the worker can consume from. Each email is routed to a
/// one and only one queue, based on the email provider. A single worker can
/// consume from multiple queues.
#[derive(Debug, Clone)]
pub enum Queue {
	Gmail,
	HotmailB2B,
	HotmailB2C,
	Yahoo,
	EverythingElse,
}

impl fmt::Display for Queue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Queue::Gmail => write!(f, "check.gmail"),
			Queue::HotmailB2B => write!(f, "check.hotmailb2b"),
			Queue::HotmailB2C => write!(f, "check.hotmailb2c"),
			Queue::Yahoo => write!(f, "check.yahoo"),
			Queue::EverythingElse => write!(f, "check.everything_else"),
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
					"check.gmail" => Ok(Queue::Gmail),
					"check.hotmailb2b" => Ok(Queue::HotmailB2B),
					"check.hotmailb2c" => Ok(Queue::HotmailB2C),
					"check.yahoo" => Ok(Queue::Yahoo),
					"check.everything_else" => Ok(Queue::EverythingElse),
					_ => Err(de::Error::unknown_variant(
						value,
						&[
							"check.gmail",
							"check.hotmailb2b",
							"check.hotmailb2c",
							"check.yahoo",
							"check.everything_else",
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
pub fn load_config() -> Result<BackendConfig, anyhow::Error> {
	let cfg = Config::builder()
		.add_source(config::File::with_name("backend_config"))
		.add_source(config::Environment::with_prefix("RCH").separator("__"))
		.build()?;

	let cfg = cfg.try_deserialize::<BackendConfig>()?;

	Ok(cfg)
}

#[cfg(test)]
mod test {
	#[test]
	fn test_load_config() {
		let cfg = super::load_config();
		assert!(cfg.is_ok());
	}
}
