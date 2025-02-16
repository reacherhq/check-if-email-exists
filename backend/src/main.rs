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
	debug!(target: LOG_TARGET, "{:#?}", config);
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
