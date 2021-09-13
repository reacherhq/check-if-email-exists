// check-if-email-exists
// Copyright (C) 2018-2021 Reacher

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

use std::{borrow::Cow, net::SocketAddr};

use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailInputProxy};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};

use crate::CONF;

/// JSON Request from POST /
#[derive(Debug, Deserialize, Serialize)]
#[deprecated(
	since = "0.8.24",
	note = "The HTTP server will be removed from the CLI, please use https://github.com/reacherhq/backend instead"
)]
pub struct PostReqBody {
	from_email: Option<String>,
	hello_name: Option<String>,
	to_emails: Vec<String>,
	proxy_host: Option<String>,
	proxy_port: Option<u16>,
}

/// Error Response from POST /
#[derive(Debug, Deserialize, Serialize)]
#[deprecated(
	since = "0.8.24",
	note = "The HTTP server will be removed from the CLI, please use https://github.com/reacherhq/backend instead"
)]
pub struct PostResError {
	error: String,
}

async fn req_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		// Serve some instructions at /
		(&Method::GET, "/") => Ok(Response::new(Body::from(
			"Send a POST request with JSON `{ \"from_email\"?: \"<email>\", \"hello_name\"?: \"<name>\", to_emails: \"<email>\" }` in the body",
		))),

		// Do email_exists check on POST /
		(&Method::POST, "/") => {
			let body = hyper::body::to_bytes(req.into_body()).await?;

			let body = match serde_json::from_slice::<PostReqBody>(&body) {
				Ok(b) => b,
				Err(err) => {
					return Ok(Response::builder()
						.status(StatusCode::BAD_REQUEST)
						.body(Body::from(format!("{}", err)))
						.expect("Response::builder with this body will not throw. qed.")
					);
				}
			};

			// Create EmailInput from body
			let mut input = CheckEmailInput::new(body.to_emails);
			input
				.set_from_email(body.from_email.unwrap_or_else(|| CONF.from_email.clone()))
				.set_hello_name(body.hello_name.unwrap_or_else(|| CONF.hello_name.clone()))
				.set_yahoo_use_api(CONF.yahoo_use_api);
			if let Some(proxy_host) = body.proxy_host.map(Cow::Owned).or_else(|| CONF.proxy_host.as_ref().map(Cow::Borrowed)) {
				input.set_proxy(CheckEmailInputProxy {
					host:proxy_host.into_owned(),
					port: body.proxy_port.unwrap_or(CONF.proxy_port)
				});
			}

			let body = check_email(&input).await;
			let body = match serde_json::to_string(&body) {
				Ok(b) => b,
				Err(err) => {
					return Ok(Response::builder()
						.status(StatusCode::BAD_REQUEST)
						.body(Body::from(format!("{}", err)))
						.expect("Response::builder with this body will not throw. qed.")
					);
				}
			};

			Ok(Response::new(Body::from(body)))
		}

		// Return the 404 Not Found for other routes.
		_ => {
			Ok(Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body(Body::empty())
				.expect("Response::builder with this body will not throw. qed.")
			)
		}
	}
}

#[deprecated(
	since = "0.8.24",
	note = "The HTTP server will be removed from the CLI, please use https://github.com/reacherhq/backend instead"
)]
pub async fn run<A: Into<SocketAddr>>(
	addr: A,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	println!("WARNING: The HTTP server is deprecated, and will be removed in v0.9.0. Please see https://github.com/reacherhq/backend for a replacement.");

	let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(req_handler)) });

	let addr = addr.into();
	let server = Server::bind(&addr).serve(service);

	println!("Listening on http://{}", addr);

	server.await?;

	Ok(())
}
