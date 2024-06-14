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
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;
use warp::Filter;

use super::errors;

/// Creates the routes for the HTTP server.
/// Making it public so that it can be used in tests/check_email.rs.
pub fn create_routes_without_bulk(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(v0::check_email::post::post_check_email())
		// The 3 following routes will 404 if o is None.
		.or(v0::bulk::post::create_bulk_job(o.clone()))
		.or(v0::bulk::get::get_bulk_job_status(o.clone()))
		.or(v0::bulk::results::get_bulk_job_result(o))
		.recover(errors::handle_rejection)
}

/// Runs the Warp server.
///
/// This function starts the Warp server and listens for incoming requests.
/// It returns a `Result` indicating whether the server started successfully or encountered an error.
pub async fn run_warp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let host = env::var("RCH_HTTP_HOST")
		.unwrap_or_else(|_| "127.0.0.1".into())
		.parse::<IpAddr>()
		.expect("Environment variable RCH_HTTP_HOST is malformed.");
	let port = env::var("PORT")
		.map(|port: String| {
			port.parse::<u16>()
				.expect("Environment variable PORT is malformed.")
		})
		.unwrap_or(8080);

	let routes = create_routes_without_bulk();

	info!(target: LOG_TARGET, host=?host,port=?port, "Server is listening");
	warp::serve(routes).run((host, port)).await;

	Ok(())
}

/// Create a DB pool.
async fn create_db() -> Result<Pool<Postgres>, sqlx::Error> {
	let pg_conn =
		env::var("DATABASE_URL").expect("Environment variable DATABASE_URL should be set");
	let pg_max_conn = env::var("RCH_DATABASE_MAX_CONNECTIONS").map_or(5, |var| {
		var.parse::<u32>()
			.expect("Environment variable RCH_DATABASE_MAX_CONNECTIONS should parse to u32")
	});

	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new()
		.max_connections(pg_max_conn)
		.connect(pg_conn.as_str())
		.await?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	Ok(pool)
}
