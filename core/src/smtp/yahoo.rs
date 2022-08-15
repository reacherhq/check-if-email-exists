// check-if-email-exists
// Copyright (C) 2018-2022 Reacher

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

use super::SmtpDetails;
use crate::util::{
	constants::LOG_TARGET, input_output::CheckEmailInput, ser_with_display::ser_with_display,
};
use async_smtp::EmailAddress;
use regex::Regex;
use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_json::error::Error as SerdeError;
use std::fmt;

const SIGNUP_PAGE: &str = "https://login.yahoo.com/account/create?specId=yidReg&lang=en-US&src=&done=https%3A%2F%2Fwww.yahoo.com&display=login";
const SIGNUP_API: &str = "https://login.yahoo.com/account/module/create?validateField=yid";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36"; // Fake one to use in API requests

/// The form inputs to pass into the HTTP request.
#[derive(Serialize)]
struct FormRequest {
	acrumb: String,
	#[serde(rename(serialize = "specId"))]
	spec_id: String,
	yid: String,
}

impl Default for FormRequest {
	fn default() -> Self {
		FormRequest {
			acrumb: "".into(),
			spec_id: "yidReg".into(),
			yid: "".into(),
		}
	}
}

impl FormRequest {
	fn new(acrumb: String, yid: String) -> Self {
		FormRequest {
			acrumb,
			yid,
			..Default::default()
		}
	}
}

/// One item in the response of the HTTP form request.
#[derive(Debug, Deserialize)]
struct FormResponseItem {
	error: String,
	name: String,
}

/// The response of the HTTP form request.
#[derive(Debug, Deserialize)]
struct FormResponse {
	errors: Vec<FormResponseItem>,
}

/// Possible errors when checking Yahoo email addresses.
#[derive(Debug, Serialize)]
pub enum YahooError {
	/// Cannot find "acrumb" field in cookie.
	NoAcrumb,
	/// Cannot find cookie in Yahoo response.
	NoCookie,
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	ReqwestError(ReqwestError),
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	SerdeError(SerdeError),
}

impl fmt::Display for YahooError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// Customize so only `x` and `y` are denoted.
		write!(f, "{:?}", self)
	}
}

impl From<ReqwestError> for YahooError {
	fn from(error: ReqwestError) -> Self {
		YahooError::ReqwestError(error)
	}
}

impl From<SerdeError> for YahooError {
	fn from(error: SerdeError) -> Self {
		YahooError::SerdeError(error)
	}
}

/// Helper function to create a reqwest client, with optional proxy.
fn create_client(input: &CheckEmailInput) -> Result<reqwest::Client, ReqwestError> {
	if let Some(proxy) = &input.proxy {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Using proxy socks://{}:{} for Yahoo API",
			input.to_email,
			proxy.host,
			proxy.port
		);

		let proxy = reqwest::Proxy::all(&format!("socks5://{}:{}", proxy.host, proxy.port))?;
		reqwest::Client::builder().proxy(proxy).build()
	} else {
		Ok(reqwest::Client::new())
	}
}

/// Use well-crafted HTTP requests to verify if a Yahoo email address exists.
/// Inspired by https://github.com/hbattat/verifyEmail.
pub async fn check_yahoo(
	to_email: &EmailAddress,
	input: &CheckEmailInput,
) -> Result<SmtpDetails, YahooError> {
	let response = create_client(input)?
		.get(SIGNUP_PAGE)
		.header("User-Agent", USER_AGENT)
		.send()
		.await?;

	// Get the cookies from the response.
	let cookies = match response.headers().get("Set-Cookie") {
		Some(x) => x,
		_ => {
			return Err(YahooError::NoCookie);
		}
	};

	let to_email = to_email.to_string();
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Yahoo 1st response: {:?}",
		to_email,
		response
	);
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Yahoo cookies: {:?}",
		to_email,
		cookies
	);

	let username = to_email
		.split('@')
		.next()
		.expect("The email is well-formed. qed.");

	// From the cookies, fetch the "acrumb" field.
	let acrumb = match cookies.to_str() {
		Ok(x) => x,
		_ => {
			return Err(YahooError::NoAcrumb);
		}
	};
	let re = Regex::new(r"s=(?P<acrumb>[^;]*)").expect("Correct regex. qed.");
	let acrumb = match re.captures(acrumb) {
		Some(x) => x,
		_ => {
			return Err(YahooError::NoAcrumb);
		}
	};

	// Mimic a real HTTP request.
	let response = create_client(input)?
		.post(SIGNUP_API)
		.header("Origin", "https://login.yahoo.com")
		.header("X-Requested-With", "XMLHttpRequest")
		.header("User-Agent", USER_AGENT)
		.header(
			"Content-type",
			"application/x-www-form-urlencoded; charset=UTF-8",
		)
		.header("Accept", "*/*")
		.header("Referer", SIGNUP_PAGE)
		.header("Accept-Encoding", "gzip, deflate, br")
		.header("Accept-Language", "en-US,en;q=0.8,ar;q=0.6")
		.header("Cookie", cookies)
		.json(&FormRequest::new(
			acrumb["acrumb"].to_string(),
			username.into(),
		))
		.send()
		.await?
		.json::<FormResponse>()
		.await?;

	log::debug!(
		target: LOG_TARGET,
		"[email={}] Yahoo 2nd response: {:?}",
		to_email,
		response
	);

	let username_exists = response
		.errors
		.iter()
		.any(|item| item.name == "yid" && item.error == "IDENTIFIER_EXISTS");

	Ok(SmtpDetails {
		can_connect_smtp: true,
		is_deliverable: username_exists,
		..Default::default()
	})
}
