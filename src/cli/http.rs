use check_if_email_exists::email_exists;
use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

struct PostBody {
	from_email: String,
	to_email: String,
}

fn check_email(req: Request<Body>) -> BoxFut {
	let mut response = Response::new(Body::empty());

	match (req.method(), req.uri().path()) {
		(&Method::GET, "/") => {
			*response.body_mut() =
				Body::from("Send a POST request with an email address in the body");
		}
		(&Method::POST, "/") => {
			*response.body_mut() =
				Body::from("Send a POST request with an email address in the body");
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
		.serve(|| service_fn(check_email))
		.map_err(|e| eprintln!("server error: {}", e));

	// Run this server for... forever!
	println!("Running server on localhost:{:?}", port);
	hyper::rt::run(server);
}
