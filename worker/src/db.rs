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

use check_if_email_exists::{CheckEmailOutput, LOG_TARGET};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{debug, info};

use crate::{check_email::WorkerPayload, config::WorkerConfig};

pub async fn save_to_db(
	pg_pool: PgPool,
	config: WorkerConfig,
	payload: &WorkerPayload,
	worker_output: Result<CheckEmailOutput, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let payload_json = serde_json::to_value(payload)?;

	match worker_output {
		Ok(output) => {
			let output_json = serde_json::to_value(output)?;

			sqlx::query!(
				r#"
				INSERT INTO reacher_results (payload, worker, result)
				VALUES ($1, $2, $3)
				RETURNING id
				"#,
				payload_json,
				config.name,
				output_json,
			)
			.fetch_one(&pg_pool)
			.await?;
		}
		Err(err) => {
			sqlx::query!(
				r#"
				INSERT INTO reacher_results (payload, worker, error)
				VALUES ($1, $2, $3)
				RETURNING id
				"#,
				payload_json,
				config.name,
				err.to_string(),
			)
			.fetch_one(&pg_pool)
			.await?;
		}
	}

	debug!(target: LOG_TARGET, email=?payload.input.to_email, "Wrote to DB");

	Ok(())
}

/// Create a DB pool.
pub async fn create_db(config: &WorkerConfig) -> Result<PgPool, sqlx::Error> {
	debug!(target: LOG_TARGET, "Connecting to DB... {db_url}", db_url=config.db.url);
	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new().connect(&config.db.url).await?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	info!(target: LOG_TARGET, table="email_results", "Connected to DB, Reacher will write results to DB");

	Ok(pool)
}
