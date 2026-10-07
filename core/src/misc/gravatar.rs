use crate::LOG_TARGET;
use md5::Digest;

const API_BASE_URL: &str = "https://www.gravatar.com/avatar/";

pub async fn check_gravatar(to_email: &str) -> Option<String> {
	let client = reqwest::Client::new();

	let mail_hash: Digest = md5::compute(to_email);

	let url = format!("{API_BASE_URL}{mail_hash:x}");

	tracing::debug!(
		target: LOG_TARGET,
		email=to_email,
		url=url,
		"Request Gravatar API"
	);

	let response = client
		.get(&url)
		// This option is necessary to return a NotFound exception instead of the default gravatar
		// image if none for the given email is found.
		.query(&[("d", "404")])
		.send()
		.await;

	tracing::debug!(
		target: LOG_TARGET,
		"[email={}] Gravatar response: {:?}",
		to_email,
		response
	);

	let response = match response {
		Ok(response) => response,
		Err(_) => return None,
	};

	match response.status() {
		reqwest::StatusCode::OK => Some(url),
		_ => None,
	}
}
