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

extern crate lambda_http;
extern crate lambda_runtime;
extern crate lettre;
extern crate serde_json;

use check_if_email_exists::{email_exists, EmailExistsError};
use lambda_http::{lambda, IntoResponse, Request, RequestExt};
use lambda_runtime::{error::HandlerError, Context};
use serde::Serialize;
use serde_json::json;
use std::borrow::Cow;

/// JSON Response
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
	address: String,
	deliverable: bool,
	domain: String,
	full_inbox: bool,
	has_catch_all: bool,
	host_exists: bool,
	valid_format: bool,
	username: String,
}
// Some ideas to add to Response
// - gravatar: Does the email have a gravatar icon?
// - dispoable: Does the email's domain come from a disposable email provider?
// - free: Is the email provider free?
// - md5_hash: MD5 hash of the email address

fn main() {
	lambda!(handler)
}

fn handler(request: Request, _: Context) -> Result<impl IntoResponse, HandlerError> {
	let query_params = request.query_string_parameters();
	if let Some(to_email) = query_params.get("to_email") {
		let from_email = query_params
			.get("from_email")
			.unwrap_or(&Cow::Borrowed("user@example.org"));

		let response = match email_exists(&from_email, &to_email) {
			Ok(details) => Response {
				address: details.address.address.to_string(),
				deliverable: details.smtp.deliverable,
				domain: details.address.domain,
				full_inbox: details.smtp.full_inbox,
				has_catch_all: details.smtp.has_catch_all,
				host_exists: details.mx.len() > 0,
				username: details.address.username,
				valid_format: details.address.valid_format,
			},
			Err(EmailExistsError::ToAddressError(_)) => Response {
				address: to_email.to_string(),
				deliverable: false,
				domain: String::new(),
				full_inbox: false,
				has_catch_all: false,
				host_exists: false,
				username: String::new(),
				valid_format: false,
			},
			Err(err) => return Ok(json!({ "error": format!("{:?}", err) })),
		};

		Ok(json!(response))
	} else {
		Ok(json!({ "error": "`to_email` is a required query param" }))
	}
}
