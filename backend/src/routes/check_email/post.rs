// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

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

//! This file implements the `POST /check_email` endpoint.

use crate::check::check_email;
use check_if_email_exists::{CheckEmailInput, CheckEmailInputProxy};
use serde::{Deserialize, Serialize};
use std::env;
use warp::Filter;

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndpointRequest {
	from_email: Option<String>,
	hello_name: Option<String>,
	proxy: Option<CheckEmailInputProxy>,
	smtp_port: Option<u16>,
	to_email: String,
}

impl From<EndpointRequest> for CheckEmailInput {
	fn from(req: EndpointRequest) -> Self {
		// Create Request for check_if_email_exists from body
		let mut input = CheckEmailInput::new(vec![req.to_email]);
		input
			.set_from_email(req.from_email.unwrap_or_else(|| {
				env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
			}))
			.set_hello_name(req.hello_name.unwrap_or_else(|| "gmail.com".into()));

		if let Some(proxy_input) = req.proxy {
			input.set_proxy(proxy_input);
		}

		if let Some(smtp_port) = req.smtp_port {
			input.set_smtp_port(smtp_port);
		}

		input
	}
}

/// The main endpoint handler that implements the logic of this route.
async fn handler(body: EndpointRequest) -> Result<impl warp::Reply, warp::Rejection> {
	// Run the future to check an email.
	Ok(warp::reply::json(&check_email(&body.into()).await))
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v0" / "check_email")
		.and(warp::post())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(handler)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
