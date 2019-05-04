extern crate lambda_http;
extern crate lambda_runtime;
extern crate serde_json;

use check_if_email_exists::email_exists;
use lambda_http::{lambda, IntoResponse, Request, RequestExt};
use lambda_runtime::{error::HandlerError, Context};
use serde_json::json;
use std::borrow::Cow;

fn main() {
	lambda!(handler)
}

fn handler(request: Request, _: Context) -> Result<impl IntoResponse, HandlerError> {
	let query_params = request.query_string_parameters();
	if let Some(to_email) = query_params.get("to_email") {
		let from_email = query_params
			.get("from_email")
			.unwrap_or(&Cow::Borrowed("user@example.org"));

		match email_exists(from_email, to_email) {
			// `serde_json::Values` impl `IntoResponse` by default, creating an
			// `application/json` response
			Some(value) => Ok(json!({ "message": value })),
			None => Ok(json!({ "error": "Can't check if email exists, sorry" })),
		}
	} else {
		Ok(json!({ "error": "`to_email` is a required query param" }))
	}
}
