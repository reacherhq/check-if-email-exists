// check_if_email_exists
// Copyright (C) 2018-2019 Amaury Martiny

// check_if_email_exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check_if_email_exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check_if_email_exists.  If not, see <http://www.gnu.org/licenses/>.

use check_if_email_exists_core::email_exists;
use futures::future;
use futures::Stream;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use serde::{Deserialize, Serialize};

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

/// JSON Request from POST /
#[derive(Debug, Deserialize, Serialize)]
pub struct PostReqBody {
	from_email: Option<String>,
	to_email: String,
}

/// Error Response from POST /
#[derive(Debug, Deserialize, Serialize)]
pub struct PostResError {
	error: String,
}
fn req_handler(req: Request<Body>) -> BoxFut {
	let mut response = Response::new(Body::empty());

	match (req.method(), req.uri().path()) {
		// Show instructions on GET /
		(&Method::GET, "/") => {
			*response.body_mut() =
				Body::from("Send a POST request with JSON `{ \"from_email\": \"<email>\", to_email: \"<email>\" }` in the body");
		}

		// Do email_exists check on POST /
		(&Method::POST, "/") => {
			let checked = req.into_body().concat2().map(move |chunk| {
				let body = &chunk.iter().cloned().collect::<Vec<u8>>();
				let body = match serde_json::from_slice::<PostReqBody>(body) {
					Ok(b) => {
						println!("YEAH! {:?}", b);
						serde_json::to_string(&email_exists(
							&b.to_email,
							&b.from_email.unwrap_or("user@example.org".into()),
						))
						.unwrap()
					}

					Err(err) => serde_json::to_string(&PostResError {
						error: format!("{}", err),
					})
					.unwrap(),
				};

				*response.body_mut() = Body::from(body);
				response
			});

			return Box::new(checked);
		}

		// 404 page on all other routes
		_ => {
			*response.status_mut() = StatusCode::NOT_FOUND;
		}
	};

	Box::new(future::ok(response))
}

pub fn run(port: u16) -> () {
	// This is our socket address...
	let addr = ([127, 0, 0, 1], port).into();

	let server = Server::bind(&addr)
		.serve(|| service_fn(req_handler))
		.map_err(|e| eprintln!("server error: {}", e));

	// Run this server for... forever!
	println!("Running server on localhost:{:?}", port);
	hyper::rt::run(server);
}
