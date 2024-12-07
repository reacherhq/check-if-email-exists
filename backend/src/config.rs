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

use crate::storage::commercial_license_trial::CommercialLicenseTrialStorage;
use crate::storage::{postgres::PostgresStorage, Storage};
use crate::worker::do_work::TaskWebhook;
use crate::worker::setup_rabbit_mq;
use anyhow::{bail, Context};
use check_if_email_exists::{
	CheckEmailInputProxy, GmailVerifMethod, HotmailB2BVerifMethod, HotmailB2CVerifMethod,
	YahooVerifMethod, LOG_TARGET,
};
use config::Config;
use lapin::Channel;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::{any::Any, collections::HashMap};
use tracing::warn;

#[derive(Debug, Default, Serialize, Deserialize)]
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
	/// Timeout for each SMTP connection, in seconds. Leaving it commented out
	/// will not set a timeout, i.e. the connection will wait indefinitely.
	pub smtp_timeout: Option<u64>,
	/// Sentry DSN to report errors to
	pub sentry_dsn: Option<String>,

	/// Worker configuration, only present if the backend is a worker.
	pub worker: WorkerConfig,

	/// Configuration on where to store the email verification results.
	pub storage: HashMap<String, StorageConfig>,

	// Internal fields, not part of the configuration.
	#[serde(skip)]
	channel: Option<Arc<Channel>>,

	#[serde(skip)]
	storages: Vec<Arc<dyn Storage>>,
}

impl BackendConfig {
	/// Get the worker configuration.
	///
	/// # Panics
	///
	/// Panics if the worker configuration is missing.
	pub fn must_worker_config(&self) -> Result<MustWorkerConfig, anyhow::Error> {
		match (
			self.worker.enable,
			&self.worker.throttle,
			&self.worker.rabbitmq,
			&self.channel,
		) {
			(true, Some(throttle), Some(rabbitmq), Some(channel)) => Ok(MustWorkerConfig {
				channel: channel.clone(),
				throttle: throttle.clone(),
				rabbitmq: rabbitmq.clone(),

				webhook: self.worker.webhook.clone(),
			}),

			(true, _, _, _) => bail!("Worker configuration is missing"),
			_ => bail!("Calling must_worker_config on a non-worker backend"),
		}
	}

	/// Attempt connection to the Postgres database and RabbitMQ. Also populates
	/// the internal `pg_pool` and `channel` fields with the connections.
	pub async fn connect(&mut self) -> Result<(), anyhow::Error> {
		if self.worker.enable && self.storage.is_empty() {
			bail!("When worker.enable is true, you must configure at least one storage to store the email verification results.");
		}

		for storage in self.storage.values() {
			match storage {
				StorageConfig::Postgres(config) => {
					let storage = PostgresStorage::new(&config.db_url, config.extra.clone())
						.await
						.with_context(|| format!("Connecting to postgres DB {}", config.db_url))?;
					self.storages.push(Arc::new(storage));
				}
				StorageConfig::CommercialLicenseTrial(config) => {
					let storage =
						CommercialLicenseTrialStorage::new(&config.db_url, config.extra.clone())
							.await
							.with_context(|| {
								format!("Connecting to postgres DB {}", config.db_url)
							})?;
					self.storages.push(Arc::new(storage));
				}
			}
		}

		let channel = if self.worker.enable {
			let rabbitmq_config = self.worker.rabbitmq.as_ref().ok_or_else(|| {
				anyhow::anyhow!("Worker configuration is missing the rabbitmq configuration")
			})?;
			let channel = setup_rabbit_mq(&self.backend_name, rabbitmq_config).await?;
			Some(Arc::new(channel))
		} else {
			None
		};
		self.channel = channel;

		Ok(())
	}

	/// Get all storages as a Vec. We don't really care about the keys in the
	/// HashMap, except for deserialize purposes.
	pub fn get_storages(&self) -> Vec<Arc<dyn Storage>> {
		self.storages.clone()
	}

	/// Get the Postgres connection pool, if at least one of the storages is a
	/// Postgres storage.
	///
	/// This is quite hacky, and it will most probably be refactored away in
	/// future versions. We however need to rethink how to do the `/v1/bulk`
	/// endpoints first.
	pub fn get_pg_pool(&self) -> Option<PgPool> {
		self.storages
			.iter()
			.find_map(|s| <dyn Any>::downcast_ref::<PostgresStorage>(s).map(|s| s.pg_pool.clone()))
	}
}

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
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

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct WorkerConfig {
	pub enable: bool,

	/// Throttle configuration for the worker.
	pub throttle: Option<ThrottleConfig>,
	pub rabbitmq: Option<RabbitMQConfig>,
	/// Optional webhook configuration to send email verification results.
	pub webhook: Option<TaskWebhook>,
}

/// Worker configuration that must be present if worker.enable is true. Used as
/// a domain type to ensure that the worker configuration is present.
#[derive(Debug, Clone)]
pub struct MustWorkerConfig {
	pub channel: Arc<Channel>,

	pub throttle: ThrottleConfig,
	pub rabbitmq: RabbitMQConfig,
	pub webhook: Option<TaskWebhook>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RabbitMQConfig {
	pub url: String,
	/// Total number of concurrent messages that the worker can process.
	pub concurrency: u16,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
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

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageConfig {
	/// Store the email verification results in the Postgres database.
	Postgres(PostgresConfig),
	/// Store the email verification results in Reacher's DB. This storage
	/// method is baked-in into the software for users of the Commercial
	/// License trial.
	CommercialLicenseTrial(PostgresConfig),
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct PostgresConfig {
	pub db_url: String,
	pub extra: Option<serde_json::Value>,
}

/// Load the worker configuration from the worker_config.toml file and from the
/// environment.
pub async fn load_config() -> Result<BackendConfig, anyhow::Error> {
	let cfg = Config::builder()
		.add_source(config::File::with_name("backend_config"))
		.add_source(config::Environment::with_prefix("RCH").separator("__"));

	let cfg = cfg.build()?.try_deserialize::<BackendConfig>()?;

	if !cfg.worker.enable && (cfg.worker.rabbitmq.is_some() || cfg.worker.throttle.is_some()) {
		warn!(target: LOG_TARGET, "worker.enable is set to false, ignoring throttling and concurrency settings.")
	}

	Ok(cfg)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;

	#[tokio::test]
	async fn test_env_vars() {
		env::set_var("RCH__BACKEND_NAME", "test-backend");
		env::set_var("RCH__STORAGE__1__POSTGRES__DB_URL", "test2");
		let cfg = load_config().await.unwrap();
		assert_eq!(cfg.backend_name, "test-backend");
		assert_eq!(
			cfg.storage.get("1").unwrap(),
			&StorageConfig::Postgres(PostgresConfig {
				db_url: "test2".to_string(),
				extra: None,
			})
		);
	}

	#[tokio::test]
	async fn test_serialize_storage_config() {
		let mut storage_config = HashMap::new();
		storage_config.insert(
			"test1",
			StorageConfig::Postgres(PostgresConfig {
				db_url: "postgres://localhost:5432/test1".to_string(),
				extra: None,
			}),
		);

		let expected = r#"[test1.postgres]
db_url = "postgres://localhost:5432/test1"
"#;

		assert_eq!(expected, toml::to_string(&storage_config).unwrap(),);
	}
}
