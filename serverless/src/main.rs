extern crate lambda_http;
extern crate lambda_runtime;
extern crate lettre;
extern crate serde_json;

use check_if_email_exists::email_exists;
use lambda_http::{lambda, IntoResponse, Request, RequestExt};
use lambda_runtime::{error::HandlerError, Context};
use lettre::EmailAddress;
use serde_json::json;
use std::borrow::Cow;
use std::str::FromStr;

/// Return HTTP response with error when there's one
macro_rules! try_or_return (
    ($res: expr) => ({
		match $res {
			Ok(value) => value,
			Err(err) => {
				return Ok(json!({ "error": format!("{:?}", err) }));
			}
		}
    })
);

fn main() {
	lambda!(handler)
}

fn handler(request: Request, _: Context) -> Result<impl IntoResponse, HandlerError> {
	let query_params = request.query_string_parameters();
	if let Some(to_email) = query_params.get("to_email") {
		let from_email = query_params
			.get("from_email")
			.unwrap_or(&Cow::Borrowed("user@example.org"));

		let from_email = try_or_return!(EmailAddress::from_str(from_email));
		let to_email = try_or_return!(EmailAddress::from_str(to_email));

		let exists = try_or_return!(email_exists(&from_email, &to_email));

		Ok(json!({ "message": exists }))
	} else {
		Ok(json!({ "error": "`to_email` is a required query param" }))
	}
}
