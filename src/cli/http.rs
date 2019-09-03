use check_if_email_exists::email_exists;
use futures::future;
use futures::Stream;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use serde::Deserialize;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

/// JSON Request from POST /
#[derive(Debug, Deserialize)]
pub struct PostReqBody {
	from_email: String,
	to_email: String,
}
fn req_handler(req: Request<Body>) -> BoxFut {
	let mut response = Response::new(Body::empty());

	match (req.method(), req.uri().path()) {
		(&Method::GET, "/") => {
			*response.body_mut() =
				Body::from("Send a POST request with JSON `{ from_email: <email>, to_email: <email> }` in the body");
		}
		(&Method::POST, "/") => {
			println!("HELLO");
			let body = req.into_body().concat2().wait().unwrap().into_bytes();
			println!("{:?}", body);
			match serde_json::from_slice::<PostReqBody>(&body) {
				Ok(b) => {
					println!("{:?}", b);
					*response.body_mut() = Body::from(
						serde_json::to_string(&email_exists(&b.from_email, &b.to_email)).unwrap(),
					);
				}
				Err(_err) => {
					println!("{:?}", _err);
					*response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
					*response.body_mut() = Body::from("Error in input");
				}
			}
		}
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
