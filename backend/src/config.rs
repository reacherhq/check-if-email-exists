use check_if_email_exists::config::ReacherConfig;
use check_if_email_exists::SentryConfig;
use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
	/// Name of the backend.
	pub backend_name: String,

	// Reacher config
	pub from_email: String,
	pub hello_name: String,
	pub webdriver_addr: String,

	// Backend-specific config
	/// Backend host
	pub http_host: String,
	/// Backend port
	pub http_port: u16,
	/// Shared secret between a trusted client and the backend.
	pub header_secret: Option<String>,
	pub throttle: ThrottleConfig,
	pub webhook: Option<WebhookConfig>,
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