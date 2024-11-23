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

use anyhow::Context;
use check_if_email_exists::LOG_TARGET;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::sync::Arc;
use tracing::{debug, info};

use crate::config::BackendConfig;

/// Create a DB pool.
pub async fn create_db(config: Arc<BackendConfig>) -> Result<PgPool, anyhow::Error> {
	let worker_config = config.must_worker_config();
	// For legacy reasons, we also support the DATABASE_URL environment variable:
	let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| worker_config.postgres.db_url.clone());

	debug!(target: LOG_TARGET, "Connecting to DB: {}", db_url);
	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new()
		.connect(&worker_config.postgres.db_url)
		.await
		.with_context(|| format!("Connecting to postgres DB {db_url}"))?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	info!(target: LOG_TARGET, table="v1_task_result", "Connected to DB, Reacher will write verification results to DB");

	Ok(pool)
}
