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
mod v1;
mod version;

use std::env;
use std::net::IpAddr;

use check_if_email_exists::LOG_TARGET;
#[cfg(feature = "worker")]
use lapin::Channel;
use tracing::info;
use warp::Filter;

use super::errors;

#[cfg(feature = "worker")]
fn create_routes_with_bulk(
	channel: Channel,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(v0::check_email::post::post_check_email())
		.or(v1::bulk::post::create_bulk_job(channel))
		.recover(errors::handle_rejection)
}

/// Creates the routes for the HTTP server.
/// Making it public so that it can be used in tests/check_email.rs.
pub fn create_routes_without_bulk(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(v0::check_email::post::post_check_email())
		.recover(errors::handle_rejection)
}

/// Runs the Warp server.
///
/// This function starts the Warp server and listens for incoming requests.
/// It returns a `Result` indicating whether the server started successfully or encountered an error.
pub async fn run_warp_server(
	#[cfg(feature = "worker")] channel: Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

	#[cfg(feature = "worker")]
	let routes = create_routes_with_bulk(channel);
	#[cfg(not(feature = "worker"))]
	let routes = create_routes_without_bulk();

	info!(target: LOG_TARGET, host=?host,port=?port, "Server is listening");
	warp::serve(routes).run((host, port)).await;

	Ok(())
}
