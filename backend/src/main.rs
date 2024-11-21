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
use lapin::options::BasicCancelOptions;
#[cfg(feature = "worker")]
use reacher_backend::worker::{create_db, run_worker, setup_rabbit_mq};
use std::sync::Arc;
use tracing::info;

use reacher_backend::config::load_config;
use reacher_backend::http::run_warp_server;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run a HTTP server using warp with bulk endpoints.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Initialize logging.
	tracing_subscriber::fmt::init();
	info!(target: LOG_TARGET, version=?CARGO_PKG_VERSION, "Running Reacher");
	let config = load_config()?;

	// Setup sentry bug tracking.
	let _guard: sentry::ClientInitGuard;
	if let Some(sentry_config) = &config.sentry {
		_guard = setup_sentry(sentry_config);
	}

	let config = Arc::new(config);

	#[cfg(feature = "worker")]
	{
		let channel = Arc::new(setup_rabbit_mq(config.clone()).await?);
		let server_future = run_warp_server(config.clone(), channel.clone());
		let worker_future = async {
			if config.worker.enable {
				let pg_pool = create_db(config.clone()).await?;
				run_worker(config.clone(), pg_pool, channel.clone()).await?;
			}
			Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
		};

		tokio::try_join!(server_future, worker_future)?;

		println!("Shutting down...");
	}

	#[cfg(not(feature = "worker"))]
	run_warp_server(config).await?;

	Ok(())
}
