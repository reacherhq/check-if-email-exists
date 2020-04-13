// check-if-email-exists
// Copyright (C) 2018-2020 Amaury Martiny

// check-if-email-exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check-if-email-exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check-if-email-exists.  If not, see <http://www.gnu.org/licenses/>.

use check_if_email_exists::{email_exists, EmailInput};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// JSON Request from POST /
#[derive(Debug, Deserialize, Serialize)]
pub struct PostReqBody {
	from_email: Option<String>,
	hello_name: Option<String>,
	to_email: String,
	// TODO Add fields for proxy
}

/// Error Response from POST /
#[derive(Debug, Deserialize, Serialize)]
pub struct PostResError {
	error: String,
}
async fn req_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		// Serve some instructions at /
		(&Method::GET, "/") => Ok(Response::new(Body::from(
			"Send a POST request with JSON `{ \"from_email\"?: \"<email>\", \"hello_name\"?: \"<name>\", to_email: \"<email>\" }` in the body",
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
			let mut input = EmailInput::new(body.to_email);
			input.from_email(body.from_email.unwrap_or_else(|| "user@example.org".into())).hello_name(body.hello_name.unwrap_or_else(|| "localhost".into()));

			let body = email_exists(&input).await;
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

pub async fn run(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// This is our socket address
	let addr = SocketAddr::new(host.parse()?, port);
	let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(req_handler)) });
	let server = Server::bind(&addr).serve(service);

	println!("Listening on http://{}", addr);

	server.await?;

	Ok(())
}
