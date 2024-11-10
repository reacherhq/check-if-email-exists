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

mod v0;
mod version;

use std::env;
use std::net::IpAddr;

use check_if_email_exists::LOG_TARGET;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use sqlxmq::JobRunnerHandle;
use tracing::info;
use warp::Filter;

use crate::config::BackendConfig;

use super::errors;

/// Creates the routes for the HTTP server.
/// Making it public so that it can be used in tests/check_email.rs.
pub fn create_routes(
	config: &BackendConfig,
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(v0::check_email::post::post_check_email(config))
		// The 3 following routes will 404 if o is None.
		.or(v0::bulk::post::create_bulk_job(o.clone()))
		.or(v0::bulk::get::get_bulk_job_status(o.clone()))
		.or(v0::bulk::results::get_bulk_job_result(o))
		.recover(errors::handle_rejection)
}

/// Runs the Warp server.
///
/// This function starts the Warp server and listens for incoming requests.
/// It returns a `Result` indicating whether the server started successfully or
/// encountered an error, as well as an optional `JobRunnerHandle` if the bulk
/// job listener is enabled. The handle can be used to stop the listener or to
/// keep it alive.
pub async fn run_warp_server(
	config: &BackendConfig,
) -> Result<Option<JobRunnerHandle>, Box<dyn std::error::Error + Send + Sync>> {
	let host = config
		.http_host
		.parse::<IpAddr>()
		.expect(format!("Invalid host: {}", config.http_host).as_str());
	// For backwards compatibility, we allow the port to be set via the
	// environment variable PORT, instead of the new configuration file. The
	// PORT environment variable takes precedence.
	let port = env::var("PORT")
		.map(|port: String| {
			port.parse::<u16>()
				.expect(format!("Invalid port: {}", port).as_str())
		})
		.unwrap_or(config.http_port);

	// Run bulk job listener.
	let is_bulk_enabled = env::var("RCH_ENABLE_BULK").unwrap_or_else(|_| "0".into()) == "1";
	let (db, runner) = if is_bulk_enabled {
		let pool = create_db().await?;
		let runner = v0::bulk::create_job_registry(&pool).await?;
		(Some(pool), Some(runner))
	} else {
		(None, None)
	};

	let routes = create_routes(config, db);

	info!(target: LOG_TARGET, host=?host,port=?port, "Server is listening");
	warp::serve(routes).run((host, port)).await;

	// Returning runner, because dropping it would stop the listener.
	Ok(runner)
}

/// Create a DB pool.
async fn create_db() -> Result<Pool<Postgres>, sqlx::Error> {
	let pg_conn = env::var("DATABASE_URL").expect("Environment variable DATABASE_URL must be set");

	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new().connect(pg_conn.as_str()).await?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	Ok(pool)
}

/// Warp filter that adds the BackendConfig to the handler.
pub fn with_config(
	config: &BackendConfig,
) -> impl Filter<Extract = (&BackendConfig,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || config)
}
