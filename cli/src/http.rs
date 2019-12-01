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

use check_if_email_exists::email_exists;
use futures::stream::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};

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
async fn req_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		// Serve some instructions at /
		(&Method::GET, "/") => Ok(Response::new(Body::from(
			"Send a POST request with JSON `{ \"from_email\": \"<email>\", to_email: \"<email>\" }` in the body",
		))),

		// Do email_exists check on POST /
        (&Method::POST, "/echo/reversed") => {
			let body = req.into_body().try_concat().await;
			let body = body.map(move |chunk| {
                chunk.iter().cloned().collect::<Vec<u8>>()

            })?;
			let body = match serde_json::from_slice::<PostReqBody>(&body) {
				Ok(b)=>b,
				_=>{
					PostReqBody {
										from_email: None,
										to_email: "a".into(),
									}
				}
			};

			println!("YEAH! {:?}", body);



			let body =
					email_exists(
							&body.to_email,
							&body.from_email.unwrap_or("user@example.org".into()),
						).await;
						let body = serde_json::to_string(&body).unwrap();


            Ok(Response::new(Body::from(body)))
        }


        // Return the 404 Not Found for other routes.
        _ => {
            Ok(Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body(Body::empty())
				.unwrap())
        }
	}
}

pub async fn run(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// This is our socket address
	let addr = ([127, 0, 0, 1], port).into();

	let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(req_handler)) });

	let server = Server::bind(&addr).serve(service);

	println!("Listening on http://{}", addr);

	server.await?;

	Ok(())
}
