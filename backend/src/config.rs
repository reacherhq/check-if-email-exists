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

use crate::storage::{postgres::PostgresStorage, StorageAdapter};
use crate::throttle::ThrottleManager;
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
use tracing::warn;

#[derive(Debug, Serialize, Deserialize)]
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
	pub storage: Option<StorageConfig>,

	/// Whether to enable the Commercial License Trial. Setting this to true
	pub commercial_license_trial: Option<CommercialLicenseTrialConfig>,

	/// Throttle configuration for all requests
	pub throttle: ThrottleConfig,

	// Internal fields, not part of the configuration.
	#[serde(skip)]
	channel: Option<Arc<Channel>>,

	#[serde(skip)]
	storage_adapter: Arc<StorageAdapter>,

	#[serde(skip)]
	throttle_manager: Arc<ThrottleManager>,
}

impl BackendConfig {
	/// Create an empty BackendConfig. This is useful for testing purposes.
	pub fn empty() -> Self {
		Self {
			backend_name: "".to_string(),
			from_email: "".to_string(),
			hello_name: "".to_string(),
			webdriver_addr: "".to_string(),
			proxy: None,
			verif_method: VerifMethodConfig::default(),
			http_host: "127.0.0.1".to_string(),
			http_port: 8080,
			header_secret: None,
			smtp_timeout: None,
			sentry_dsn: None,
			worker: WorkerConfig::default(),
			storage: Some(StorageConfig::Noop),
			commercial_license_trial: None,
			throttle: ThrottleConfig::new_without_throttle(),
			channel: None,
			storage_adapter: Arc::new(StorageAdapter::Noop),
			throttle_manager: Arc::new(
				ThrottleManager::new(ThrottleConfig::new_without_throttle()),
			),
		}
	}

	/// Get the worker configuration.
	///
	/// # Panics
	///
	/// Panics if the worker configuration is missing.
	pub fn must_worker_config(&self) -> Result<MustWorkerConfig, anyhow::Error> {
		match (self.worker.enable, &self.worker.rabbitmq, &self.channel) {
			(true, Some(rabbitmq), Some(channel)) => Ok(MustWorkerConfig {
				channel: channel.clone(),
				rabbitmq: rabbitmq.clone(),
				webhook: self.worker.webhook.clone(),
			}),

			(true, _, _) => bail!("Worker configuration is missing"),
			_ => bail!("Calling must_worker_config on a non-worker backend"),
		}
	}

	/// Attempt connection to the Postgres database and RabbitMQ. Also populates
	/// the internal `pg_pool` and `channel` fields with the connections.
	pub async fn connect(&mut self) -> Result<(), anyhow::Error> {
		match &self.storage {
			Some(StorageConfig::Postgres(config)) => {
				let storage = PostgresStorage::new(&config.db_url, config.extra.clone())
					.await
					.with_context(|| format!("Connecting to postgres DB {}", config.db_url))?;

				self.storage_adapter = Arc::new(StorageAdapter::Postgres(storage));
			}
			_ => {
				self.storage_adapter = Arc::new(StorageAdapter::Noop);
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

		// Initialize throttle manager
		self.throttle_manager = Arc::new(ThrottleManager::new(self.throttle.clone()));

		Ok(())
	}

	/// Get all storages as a Vec. We don't really care about the keys in the
	/// HashMap, except for deserialize purposes.
	pub fn get_storage_adapter(&self) -> Arc<StorageAdapter> {
		self.storage_adapter.clone()
	}

	/// Get the Postgres connection pool, if the storage is Postgres.
	pub fn get_pg_pool(&self) -> Option<PgPool> {
		match self.storage_adapter.as_ref() {
			StorageAdapter::Postgres(storage) => Some(storage.pg_pool.clone()),
			StorageAdapter::Noop => None,
		}
	}

	pub fn get_throttle_manager(&self) -> Arc<ThrottleManager> {
		self.throttle_manager.clone()
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
	pub rabbitmq: Option<RabbitMQConfig>,
	/// Optional webhook configuration to send email verification results.
	pub webhook: Option<TaskWebhook>,
}

/// Worker configuration that must be present if worker.enable is true. Used as
/// a domain type to ensure that the worker configuration is present.
#[derive(Debug, Clone)]
pub struct MustWorkerConfig {
	pub channel: Arc<Channel>,
	pub rabbitmq: RabbitMQConfig,
	pub webhook: Option<TaskWebhook>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RabbitMQConfig {
	pub url: String,
	/// Total number of concurrent messages that the worker can process.
	pub concurrency: u16,
}

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageConfig {
	/// Store the email verification results in the Postgres database.
	Postgres(PostgresConfig),
	/// Don't store the email verification results.
	#[default]
	Noop,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct PostgresConfig {
	pub db_url: String,
	pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct CommercialLicenseTrialConfig {
	pub api_token: String,
	pub url: String,
}

/// Load the worker configuration from the worker_config.toml file and from the
/// environment.
pub async fn load_config() -> Result<BackendConfig, anyhow::Error> {
	let cfg = Config::builder()
		.add_source(config::File::with_name("backend_config"))
		.add_source(config::Environment::with_prefix("RCH").separator("__"));

	let cfg = cfg.build()?.try_deserialize::<BackendConfig>()?;

	if !cfg.worker.enable && cfg.worker.rabbitmq.is_some() {
		warn!(target: LOG_TARGET, "worker.enable is set to false, ignoring concurrency settings.")
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
		env::set_var("RCH__STORAGE__POSTGRES__DB_URL", "test2");
		let cfg = load_config().await.unwrap();
		assert_eq!(cfg.backend_name, "test-backend");
		assert_eq!(
			cfg.storage,
			Some(StorageConfig::Postgres(PostgresConfig {
				db_url: "test2".to_string(),
				extra: None,
			}))
		);
	}

	#[tokio::test]
	async fn test_serialize_storage_config() {
		let storage_config = StorageConfig::Postgres(PostgresConfig {
			db_url: "postgres://localhost:5432/test1".to_string(),
			extra: None,
		});

		let expected = r#"[postgres]
db_url = "postgres://localhost:5432/test1"
"#;

		assert_eq!(expected, toml::to_string(&storage_config).unwrap(),);
	}
}
