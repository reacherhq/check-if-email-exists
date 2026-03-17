use crate::LOG_TARGET;
use reqwest::Client;

const MAIN_API_URL: &str = "https://haveibeenpwned.com/api/v3/";

/// Check if the email has been found in any breach or paste using the
/// HaveIBeenPwned API.
/// This function will return the number of times the email has been found in
/// any breach.
pub async fn check_haveibeenpwned(to_email: &str, api_key: Option<String>) -> Option<bool> {
	let client = Client::new();
	let url = format!(
		"{}breachedaccount/{}?truncateResponse=false",
		MAIN_API_URL, to_email
	);

	let request = client
		.get(&url)
		.header("User-Agent", "reacher")
		.header("hibp-api-key", api_key.unwrap_or_default())
		.send()
		.await;

	match request {
		Ok(response) => {
			if response.status().is_success() {
				let breaches: Vec<serde_json::Value> = response.json().await.unwrap_or_default();
				tracing::debug!(
					target: LOG_TARGET,
					breach_count=breaches.len(),
					"HaveIBeenPwned check completed"
				);
				Some(!breaches.is_empty())
			} else if response.status() == reqwest::StatusCode::NOT_FOUND {
				Some(false)
			} else {
				tracing::error!(
					target: LOG_TARGET,
					status = %response.status(),
					"Error checking HaveIBeenPwned"
				);
				None
			}
		}
		Err(e) => {
			tracing::error!(
				target: LOG_TARGET,
				error=?e,
				"Error checking HaveIBeenPwned"
			);
			None
		}
	}
}
