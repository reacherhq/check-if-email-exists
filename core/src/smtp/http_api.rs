use crate::util::input_output::CheckEmailInput;
use reqwest::Error as ReqwestError;

/// Helper function to create a reqwest client, with optional proxy.
pub fn create_client(
	_input: &CheckEmailInput,
	_api_name: &str,
) -> Result<reqwest::Client, ReqwestError> {
	// TODO: Allow proxying for HTTP API requests.
	Ok(reqwest::Client::new())
}
