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
use reacher_worker::config::load_config;
use reacher_worker::db::create_db;
use reacher_worker::worker::run_worker;
use reacher_worker::CARGO_PKG_VERSION;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Initialize logging.
	tracing_subscriber::fmt::init();
	info!(target: LOG_TARGET, version=?CARGO_PKG_VERSION, "Running Reacher Worker");

	// Setup sentry bug tracking.
	let _guard: sentry::ClientInitGuard = setup_sentry();

	let config = load_config()?;
	let pg_pool = create_db(&config).await?;

	run_worker(config, pg_pool).await
}
