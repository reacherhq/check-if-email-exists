//! Main entry point of the `reacher_backend` binary. It has two `main`
//! functions, depending on whether the `bulk` feature is enabled or not.

use check_if_email_exists::{setup_sentry, LOG_TARGET};
use reacher_backend::config::load_config;
use reacher_backend::http::run_warp_server;
use reacher_backend::worker::run_worker;
use std::sync::Arc;
use tracing::{debug, info};

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run a HTTP server using warp with bulk endpoints.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
	// Initialize logging.
	tracing_subscriber::fmt::init();

	info!(target: LOG_TARGET, version=?CARGO_PKG_VERSION, "Running Reacher");
	let mut config = load_config().await?;
	config.connect().await?;

	// SECURITY: Don't log full config as it likely contains secrets (DB_URL, etc)
	debug!(target: LOG_TARGET, "{:#?}", config.get_verif_method());

	// Setup sentry bug tracking.
	let _guard: sentry::ClientInitGuard;
	if let Some(sentry_config) = &config.sentry_dsn {
		_guard = setup_sentry(sentry_config);
	}

	let config = Arc::new(config);

	let server_future = run_warp_server(Arc::clone(&config));
	let worker_future = async {
		if config.worker.enable {
			run_worker(config).await?;
		}
		Ok(())
	};

	tokio::try_join!(server_future, worker_future)?;

	info!("Shutting down...");

	Ok(())
}
