use super::YahooError;
use crate::LOG_TARGET;
use crate::{
	smtp::{http_api::create_client, SmtpDetails},
	util::input_output::CheckEmailInput,
};
use regex::Regex;
use reqwest::header::{HeaderMap, SET_COOKIE};
use serde::{Deserialize, Serialize};

const SPEC_ID: &str = "yidregsimplified";
const SIGNUP_PAGE: &str = "https://login.yahoo.com/account/create?lang=en-US&src=ym&done=https%3A%2F%2Fmail.yahoo.com%2F&specId=yidregsimplified";
const VALIDATE_API: &str = "https://login.yahoo.com/account/create/validate?lang=en-US&src=ym&done=https%3A%2F%2Fmail.yahoo.com%2F&specId=yidregsimplified&validateField=userId";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";
/// Session cookie set by the signup page. Without it the validate endpoint
/// redirects to `/account/challenge/fail`.
const SESSION_COOKIE: &str = "AS";

/// The form inputs to pass into the HTTP request.
#[derive(Serialize)]
struct FormRequest<'a> {
	acrumb: &'a str,
	crumb: &'a str,
	#[serde(rename = "sessionIndex")]
	session_index: &'a str,
	#[serde(rename = "specId")]
	spec_id: &'a str,
	#[serde(rename = "userId")]
	user_id: &'a str,
	#[serde(rename = "yidDomain")]
	yid_domain: &'a str,
}

#[derive(Debug, Deserialize)]
struct FieldError {
	id: String,
}

#[derive(Debug, Deserialize)]
struct UserIdField {
	error: Option<FieldError>,
}

#[derive(Debug, Deserialize)]
struct Fields {
	#[serde(rename = "userId")]
	user_id: UserIdField,
}

/// The response of the validate endpoint: either the validated form fields,
/// or a redirect when Yahoo rejects the request (bot challenge, rate limit,
/// missing session).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ValidateResponse {
	Fields { fields: Fields },
	Redirect { redirect: String },
}

/// Use well-crafted HTTP requests to verify if a Yahoo email address exists,
/// by asking the signup form whether the username is already taken.
pub async fn check_api(to_email: &str, input: &CheckEmailInput) -> Result<SmtpDetails, YahooError> {
	let client = create_client(input, "yahoo")?;

	let res = client
		.get(SIGNUP_PAGE)
		.header("User-Agent", USER_AGENT)
		.send()
		.await?;

	let cookies = cookie_header(res.headers()).ok_or(YahooError::NoCookie)?;

	tracing::debug!(
		target: LOG_TARGET,
		email=to_email,
		"Yahoo succesfully got cookies after response"
	);

	let body = res.text().await?;
	let session_index = hidden_input(&body, "sessionIndex").ok_or(YahooError::NoSessionIndex)?;
	let acrumb = hidden_input(&body, "acrumb").ok_or(YahooError::NoAcrumb)?;
	// Not required by Yahoo at the moment, but sent by the browser.
	let crumb = hidden_input(&body, "crumb").unwrap_or_default();

	let username = to_email
		.split('@')
		.next()
		.expect("The email is well-formed. qed.");

	let res = client
		.post(VALIDATE_API)
		.header("Origin", "https://login.yahoo.com")
		.header("Referer", SIGNUP_PAGE)
		.header("X-Requested-With", "XMLHttpRequest")
		.header("User-Agent", USER_AGENT)
		.header("Accept", "*/*")
		.header("Accept-Language", "en-US,en;q=0.9")
		.header("Cookie", &cookies)
		.form(&FormRequest {
			acrumb: &acrumb,
			crumb: &crumb,
			session_index: &session_index,
			spec_id: SPEC_ID,
			user_id: username,
			yid_domain: "yahoo.com",
		})
		.send()
		.await?
		.json::<ValidateResponse>()
		.await?;

	tracing::debug!(
		target: LOG_TARGET,
		email=to_email,
		"Yahoo 2nd response: {:?}",
		res
	);

	Ok(SmtpDetails {
		can_connect_smtp: true,
		is_deliverable: username_exists(res)?,
		..Default::default()
	})
}

/// Build a `Cookie` request header from the `Set-Cookie` response headers.
/// Returns `None` if Yahoo did not set the session cookie.
fn cookie_header(headers: &HeaderMap) -> Option<String> {
	let pairs: Vec<&str> = headers
		.get_all(SET_COOKIE)
		.iter()
		.filter_map(|v| v.to_str().ok())
		.filter_map(|v| v.split(';').next())
		.map(str::trim)
		.collect();

	let has_session = pairs
		.iter()
		.any(|pair| pair.split('=').next() == Some(SESSION_COOKIE));

	has_session.then(|| pairs.join("; "))
}

/// Extract the value of `<input type="hidden" name="{name}" value="...">`,
/// whatever the attribute order.
fn hidden_input(body: &str, name: &str) -> Option<String> {
	let tag = Regex::new(&format!(
		r#"<input[^>]*\bname="{}"[^>]*>"#,
		regex::escape(name)
	))
	.expect("Correct regex. qed.");
	let value = Regex::new(r#"\bvalue="(?P<value>[^"]*)""#).expect("Correct regex. qed.");

	let tag = tag.find(body)?.as_str();
	value.captures(tag).map(|c| c["value"].to_string())
}

/// Interpret the `userId` validation result.
fn username_exists(res: ValidateResponse) -> Result<bool, YahooError> {
	let fields = match res {
		ValidateResponse::Fields { fields } => fields,
		ValidateResponse::Redirect { redirect } => {
			return Err(YahooError::Rejected(redirect));
		}
	};

	match fields.user_id.error {
		None => Ok(false),
		Some(e) if e.id == "IDENTIFIER_EXISTS" || e.id == "IDENTIFIER_NOT_AVAILABLE" => Ok(true),
		// Signup rules (length, allowed characters...) are stricter than for
		// legacy accounts, so a rule violation says nothing about existence.
		Some(e) => Err(YahooError::UnexpectedUserIdError(e.id)),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use reqwest::header::HeaderValue;

	#[test]
	fn hidden_input_name_before_value() {
		let body = r#"<input type="hidden" name="sessionIndex" value="QQ--"/><input type="hidden" name="acrumb" value="m95sRVXY"/><input type="hidden" name="crumb" value="Pwf0Fl9eovX821o/0RFFyw"/>"#;
		assert_eq!(hidden_input(body, "sessionIndex").as_deref(), Some("QQ--"));
		assert_eq!(hidden_input(body, "acrumb").as_deref(), Some("m95sRVXY"));
		assert_eq!(
			hidden_input(body, "crumb").as_deref(),
			Some("Pwf0Fl9eovX821o/0RFFyw")
		);
	}

	#[test]
	fn hidden_input_value_before_name() {
		let body = r#"<input type="hidden" value="Qg--" name="sessionIndex">"#;
		assert_eq!(hidden_input(body, "sessionIndex").as_deref(), Some("Qg--"));
	}

	#[test]
	fn hidden_input_missing() {
		assert_eq!(hidden_input("<html></html>", "sessionIndex"), None);
	}

	#[test]
	fn cookie_header_requires_session_cookie() {
		let mut headers = HeaderMap::new();
		headers.append(SET_COOKIE, HeaderValue::from_static("B=abc; Path=/"));
		assert_eq!(cookie_header(&headers), None);

		headers.append(
			SET_COOKIE,
			HeaderValue::from_static("AS=v=1&s=m95sRVXY&d=A6b; Domain=login.yahoo.com; Secure"),
		);
		assert_eq!(
			cookie_header(&headers).as_deref(),
			Some("B=abc; AS=v=1&s=m95sRVXY&d=A6b")
		);
	}

	fn parse(json: &str) -> ValidateResponse {
		serde_json::from_str(json).expect("valid fixture")
	}

	#[test]
	fn existing_username() {
		let res = parse(
			r#"{"specId":"yidregsimplified","fields":{"userId":{"value":"vitalik","error":{"id":"IDENTIFIER_EXISTS"}},"password":{"error":{"id":"ERROR_MISSING_PASSWORD"}}}}"#,
		);
		assert!(username_exists(res).unwrap());
	}

	#[test]
	fn available_username() {
		let res = parse(r#"{"fields":{"userId":{"value":"qp7zk2mxw9vrt4"}}}"#);
		assert!(!username_exists(res).unwrap());
	}

	#[test]
	fn rule_violation_is_inconclusive() {
		let res =
			parse(r#"{"fields":{"userId":{"value":"ab","error":{"id":"LENGTH_TOO_SHORT"}}}}"#);
		assert!(matches!(
			username_exists(res),
			Err(YahooError::UnexpectedUserIdError(id)) if id == "LENGTH_TOO_SHORT"
		));
	}

	#[test]
	fn challenge_redirect_is_an_error() {
		let res = parse(r#"{"redirect":"/account/challenge/fail?specId=yidregsimplified"}"#);
		assert!(matches!(username_exists(res), Err(YahooError::Rejected(_))));
	}

	// Hits the live Yahoo signup form. Yahoo rate-limits bursts of requests
	// per IP, so this stays out of the default test run.
	#[tokio::test]
	#[ignore = "Calls the live Yahoo signup API"]
	async fn test_yahoo_address() {
		crate::initialize_crypto_provider();
		let input = CheckEmailInput::default();

		let res = check_api("johnsmith@yahoo.com", &input).await.unwrap();
		assert!(res.is_deliverable);

		let res = check_api("qp7zk2mxw9vrt4@yahoo.com", &input).await.unwrap();
		assert!(!res.is_deliverable);
	}
}
