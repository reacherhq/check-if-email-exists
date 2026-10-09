//! This file implements the `GET /version` endpoint.

use crate::CARGO_PKG_VERSION;
use serde::{Deserialize, Serialize};
use warp::Filter;

/// Endpoint response body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct EndpointVersion {
	version: String,
}

/// Create the `GET /version` endpoint.
pub fn get_version() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	warp::path("version").and(warp::get()).map(|| {
		warp::reply::json(&EndpointVersion {
			version: CARGO_PKG_VERSION.into(),
		})
	})
}

#[cfg(test)]
mod tests {
	use super::get_version;
	use crate::CARGO_PKG_VERSION;
	use warp::http::StatusCode;
	use warp::test::request;

	#[tokio::test]
	async fn test_get_version() {
		let resp = request()
			.path("/version")
			.method("GET")
			.reply(&get_version())
			.await;

		assert_eq!(resp.status(), StatusCode::OK);
		assert_eq!(
			resp.body(),
			format!("{{\"version\":\"{CARGO_PKG_VERSION}\"}}").as_str()
		);
	}
}
