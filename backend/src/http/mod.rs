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

use check_if_email_exists::LOG_TARGET;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use sqlxmq::JobRunnerHandle;
use std::env;
use std::net::IpAddr;
use std::sync::Arc;
use tracing::info;
use warp::Filter;
use warp::{http, reject};

use crate::config::BackendConfig;

pub use v0::check_email::post::CheckEmailRequest;

/// Creates the routes for the HTTP server.
/// Making it public so that it can be used in tests/check_email.rs.
pub fn create_routes(
	config: Arc<BackendConfig>,
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(v0::check_email::post::post_check_email(config.clone()))
		// The 3 following routes will 404 if o is None.
		.or(v0::bulk::post::create_bulk_job(config, o.clone()))
		.or(v0::bulk::get::get_bulk_job_status(o.clone()))
		.or(v0::bulk::results::get_bulk_job_result(o))
		.recover(handle_rejection)
}

/// Runs the Warp server.
///
/// This function starts the Warp server and listens for incoming requests.
/// It returns a `Result` indicating whether the server started successfully or
/// encountered an error, as well as an optional `JobRunnerHandle` if the bulk
/// job listener is enabled. The handle can be used to stop the listener or to
/// keep it alive.
pub async fn run_warp_server(
	config: Arc<BackendConfig>,
) -> Result<Option<JobRunnerHandle>, Box<dyn std::error::Error + Send + Sync>> {
	let host = config
		.http_host
		.parse::<IpAddr>()
		.unwrap_or_else(|_| panic!("Invalid host: {}", config.http_host));
	// For backwards compatibility, we allow the port to be set via the
	// environment variable PORT, instead of the new configuration file. The
	// PORT environment variable takes precedence.
	let port = env::var("PORT")
		.map(|port: String| {
			port.parse::<u16>()
				.unwrap_or_else(|_| panic!("Invalid port: {}", port))
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

	let routes = create_routes(config.clone(), db);

	info!(target: LOG_TARGET, host=?host,port=?port, "Server is listening");
	warp::serve(routes).run((host, port)).await;

	// Returning runner, because dropping it would stop the listener.
	Ok(runner)
}

/// Create a DB pool for the deprecated /v0/bulk endpoints.
async fn create_db() -> Result<Pool<Postgres>, sqlx::Error> {
	let pg_conn = env::var("DATABASE_URL").expect("Environment variable DATABASE_URL must be set");

	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new().connect(pg_conn.as_str()).await?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	Ok(pool)
}

/// The header which holds the Reacher backend secret.
pub const REACHER_SECRET_HEADER: &str = "x-reacher-secret";

/// Warp filter to check that the header secret is correct, if the header is
/// set in the config.
pub fn check_header(config: Arc<BackendConfig>) -> warp::filters::BoxedFilter<()> {
	if let Some(secret) = config.header_secret.clone() {
		if secret.is_empty() {
			return warp::any().boxed();
		}

		let secret: &'static str = Box::leak(Box::new(secret));

		warp::header::exact(REACHER_SECRET_HEADER, secret).boxed()
	} else {
		warp::any().boxed()
	}
}

/// Warp filter that adds the BackendConfig to the handler.
pub fn with_config(
	config: Arc<BackendConfig>,
) -> impl Filter<Extract = (Arc<BackendConfig>,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || config.clone())
}

/// Struct describing an error response.
#[derive(Serialize, Debug)]
pub struct ReacherResponseError {
	#[serde(skip)]
	pub code: http::StatusCode,
	pub message: String,
}

impl reject::Reject for ReacherResponseError {}

/// This function receives a `Rejection` and tries to return a custom value,
/// otherwise simply passes the rejection along.
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
	if let Some(err) = err.find::<ReacherResponseError>() {
		Ok((warp::reply::with_status(warp::reply::json(err), err.code),))
	} else {
		Err(err)
	}
}
