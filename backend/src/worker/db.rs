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

use std::env;

use check_if_email_exists::{CheckEmailOutput, LOG_TARGET};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::{debug, info};

pub async fn save_to_db(
	conn_pool: Pool<Postgres>,
	output: &CheckEmailOutput,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let output_json = serde_json::to_value(output)?;
	let is_reachable = format!("{:?}", output.is_reachable);
	sqlx::query!(
		r#"
		INSERT INTO email_results (email, is_reachable, full_result)
		VALUES ($1, $2, $3)
		RETURNING id
		"#,
		output.input,
		is_reachable,
		output_json
	)
	.fetch_one(&conn_pool)
	.await?;

	debug!(target: LOG_TARGET, email=?output.input, is_reachable=?is_reachable, "Wrote to DB");

	Ok(())
}

/// Create a DB pool.
pub async fn create_db() -> Result<Option<Pool<Postgres>>, sqlx::Error> {
	let pg_conn = env::var("DATABASE_URL").ok();
	let pg_max_conn = env::var("RCH_DATABASE_MAX_CONNECTIONS").map_or(5, |var| {
		var.parse::<u32>()
			.expect("Environment variable RCH_DATABASE_MAX_CONNECTIONS should parse to u32")
	});

	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = match pg_conn {
		Some(pg_conn_str) => {
			let pool = PgPoolOptions::new()
				.max_connections(pg_max_conn)
				.connect(pg_conn_str.as_str())
				.await?;

			sqlx::migrate!("./migrations").run(&pool).await?;

			info!(target: LOG_TARGET, table="email_results", "Connected to DB, Reacher will write results to DB");

			Some(pool)
		}
		None => None,
	};

	Ok(pool)
}
