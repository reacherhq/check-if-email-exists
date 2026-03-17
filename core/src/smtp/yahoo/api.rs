use super::YahooError;
use crate::LOG_TARGET;
use crate::{
	smtp::{http_api::create_client, SmtpDetails},
	util::input_output::CheckEmailInput,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

const SIGNUP_PAGE: &str = "https://login.yahoo.com/account/create?specId=yidReg&lang=en-US&src=&done=https%3A%2F%2Fwww.yahoo.com&display=login";
const SIGNUP_API: &str = "https://login.yahoo.com/account/module/create?validateField=yid";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36"; // Fake one to use in API requests

/// The form inputs to pass into the HTTP request.
#[derive(Serialize)]
struct FormRequest {
	acrumb: String,
	#[serde(rename(serialize = "sessionIndex"))]
	session_index: String,
	#[serde(rename(serialize = "specId"))]
	spec_id: String,
	#[serde(rename(serialize = "userId"))]
	user_id: String,
}

impl Default for FormRequest {
	fn default() -> Self {
		FormRequest {
			acrumb: "".into(),
			session_index: "".into(),
			spec_id: "yidReg".into(),
			user_id: "".into(),
		}
	}
}

impl FormRequest {
	fn new(acrumb: String, session_index: String, user_id: String) -> Self {
		FormRequest {
			acrumb,
			session_index,
			user_id,
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

/// Use well-crafted HTTP requests to verify if a Yahoo email address exists.
/// Inspired by https://github.com/hbattat/verifyEmail.
pub async fn check_api(to_email: &str, input: &CheckEmailInput) -> Result<SmtpDetails, YahooError> {
	let res = create_client(input, "yahoo")?
		.get(SIGNUP_PAGE)
		.header("User-Agent", USER_AGENT)
		.send()
		.await?;

	// Get the cookies from the response.
	let cookies = match res.headers().get("Set-Cookie") {
		Some(x) => x.to_owned(),
		_ => {
			return Err(YahooError::NoCookie);
		}
	};

	tracing::debug!(
		target: LOG_TARGET,
		email=to_email,
		"Yahoo succesfully got cookies after response"
	);

	let body = res.text().await?;

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
	let re = Regex::new(r"s=(?P<acrumb>[^;]*)&d").expect("Correct regex. qed.");
	let acrumb = match re.captures(acrumb) {
		Some(x) => x,
		_ => {
			return Err(YahooError::NoAcrumb);
		}
	};

	let re =
		Regex::new(r#"<input type="hidden" value="(?P<sessionIndex>.*)" name="sessionIndex">"#)
			.expect("Correct regex. qed");
	let session_index = match re.captures(&body) {
		Some(y) => y,
		_ => {
			return Err(YahooError::NoSessionIndex);
		}
	};

	// Mimic a real HTTP request.
	let res = create_client(input, "yahoo")?
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
		.header("Cookie", &cookies)
		.json(&FormRequest::new(
			acrumb["acrumb"].to_string(),
			session_index["sessionIndex"].to_string(),
			username.into(),
		))
		.send()
		.await?
		.json::<FormResponse>()
		.await?;

	tracing::debug!(
		target: LOG_TARGET,
		email=to_email,
		"Yahoo 2nd response: {:?}",
		res
	);

	let username_exists = res.errors.iter().any(|item| {
		item.name == "userId"
			&& (item.error == "IDENTIFIER_NOT_AVAILABLE" || item.error == "IDENTIFIER_EXISTS")
	});

	Ok(SmtpDetails {
		can_connect_smtp: true,
		is_deliverable: username_exists,
		..Default::default()
	})
}
