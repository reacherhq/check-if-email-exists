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
			format!("{{\"version\":\"{}\"}}", CARGO_PKG_VERSION).as_str()
		);
	}
}
