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
extern crate serde_json;

use check_if_email_exists_core::email_exists;
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

		let response = email_exists(&to_email, &from_email);

		Ok(json!(response))
	} else {
		Ok(json!({ "error": "`to_email` is a required query param" }))
	}
}
