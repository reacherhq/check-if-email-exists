use check_if_email_exists::CheckEmailInput;
use serde::Deserialize;

pub mod worker;

#[derive(Debug, Deserialize)]
pub struct CheckEmailPayload {
	pub input: CheckEmailInput,
	pub webhook: Option<CheckEmailWebhook>,
}

#[derive(Debug, Deserialize)]
pub struct CheckEmailWebhook {
	pub url: String,
	pub extra: serde_json::Value,
}
