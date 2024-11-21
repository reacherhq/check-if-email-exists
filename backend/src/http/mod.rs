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

mod error;
mod v0;
mod v1;
mod version;

use check_if_email_exists::LOG_TARGET;
#[cfg(feature = "worker")]
use lapin::Channel;
use sqlx::PgPool;
use sqlxmq::JobRunnerHandle;
use std::env;
use std::net::IpAddr;
use std::sync::Arc;
use tracing::info;
use warp::Filter;

use crate::config::BackendConfig;
use error::handle_rejection;
pub use error::ReacherResponseError;
pub use v0::check_email::post::CheckEmailRequest;

pub fn create_routes(
	config: Arc<BackendConfig>,
	pg_pool: PgPool,
	#[cfg(feature = "worker")] channel: Arc<Channel>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	let t = version::get::get_version()
		.or(v0::check_email::post::post_check_email(Arc::clone(&config)))
		// The 3 following routes will 404 if o is None.
		.or(v0::bulk::post::create_bulk_job(
			Arc::clone(&config),
			Some(pg_pool.clone()),
		))
		.or(v0::bulk::get::get_bulk_job_status(Some(pg_pool.clone())))
		.or(v0::bulk::results::get_bulk_job_result(Some(
			pg_pool.clone(),
		)));

	#[cfg(feature = "worker")]
	{
		t.or(v1::check_email::post::v1_check_email(
			Arc::clone(&config),
			channel.clone(),
		))
		.or(v1::bulk::post::v1_create_bulk_job(
			config,
			channel,
			pg_pool.clone(),
		))
		.or(v1::bulk::get_summary::v1_get_bulk_job_summary(
			pg_pool.clone(),
		))
		.or(v1::bulk::get_results::v1_get_bulk_job_results(pg_pool))
		.recover(handle_rejection)
	}

	#[cfg(not(feature = "worker"))]
	{
		t.recover(handle_rejection)
	}
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
	#[cfg(feature = "worker")] channel: Arc<Channel>,
	pg_pool: PgPool,
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

	let routes = create_routes(Arc::clone(&config), pg_pool.clone(), channel);

	// Run v0 bulk job listener.
	let is_bulk_enabled = env::var("RCH_ENABLE_BULK").unwrap_or_else(|_| "0".into()) == "1";
	let runner = if is_bulk_enabled {
		let runner = v0::bulk::create_job_registry(&pg_pool).await?;
		Some(runner)
	} else {
		None
	};

	info!(target: LOG_TARGET, host=?host,port=?port, "Server is listening");
	warp::serve(routes).run((host, port)).await;

	// Returning runner, because dropping it would stop the listener.
	Ok(runner)
}

/// Warp filter to add the database pool to the handler.
pub fn with_db(
	o: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || o.clone())
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
	warp::any().map(move || Arc::clone(&config))
}
