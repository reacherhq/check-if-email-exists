mod gravatar;
use crate::haveibeenpwned::check_haveibeenpwned;
use crate::syntax::SyntaxDetails;
use gravatar::check_gravatar;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, default::Default};
use thiserror::Error;

const ROLE_ACCOUNTS: &str = include_str!("./roles.txt");
const FREE_EMAIL_PROVIDERS: &str = include_str!("./b2c.txt");

// Lazy static initialization of domain sets
static ROLE_ACCOUNTS_SET: Lazy<HashSet<String>> = Lazy::new(|| load_str_as_hashset(ROLE_ACCOUNTS));
static FREE_EMAIL_PROVIDERS_SET: Lazy<HashSet<String>> =
	Lazy::new(|| load_str_as_hashset(FREE_EMAIL_PROVIDERS));

// Function to load a file with `\n`-separated lines into a HashSet.
fn load_str_as_hashset(file_content: &str) -> HashSet<String> {
	file_content
		.lines()
		.map(|line| line.trim().to_string())
		.collect()
}

/// Miscellaneous details about the email address.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MiscDetails {
	/// Is this a DEA (disposable email account)?
	pub is_disposable: bool,
	/// Is this email a role-based account?
	pub is_role_account: bool,
	/// Is this email a B2C email address?
	pub is_b2c: bool,
	/// If set, the gravatar URL for this email address.
	pub gravatar_url: Option<String>,
	/// Is this email address listed in the haveibeenpwned database for
	/// previous breaches?
	pub haveibeenpwned: Option<bool>,
}

/// Error occurred connecting to this email server via SMTP. Right now this
/// enum has no variant, as `check_misc` cannot fail. But putting a placeholder
/// right now to avoid future breaking changes.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum MiscError {}

/// Fetch misc details about the email address, such as whether it's disposable.
pub async fn check_misc(
	syntax: &SyntaxDetails,
	cfg_check_gravatar: bool,
	haveibeenpwned_api_key: Option<String>,
) -> MiscDetails {
	let address = syntax
		.address
		.as_ref()
		.expect("We already checked that the syntax was valid. qed.")
		.to_string();

	let mut gravatar_url: Option<String> = None;

	if cfg_check_gravatar {
		gravatar_url = check_gravatar(address.as_ref()).await;
	}

	let mut haveibeenpwned: Option<bool> = None;

	if haveibeenpwned_api_key.is_some() {
		haveibeenpwned = check_haveibeenpwned(address.as_ref(), haveibeenpwned_api_key).await;
	}

	MiscDetails {
		// mailchecker::is_valid checks also if the syntax is valid. But if
		// we're here, it means we're sure the syntax is valid, so is_valid
		// actually will only check if it's disposable.
		is_disposable: !mailchecker::is_valid(address.as_ref()),
		is_role_account: ROLE_ACCOUNTS_SET.contains(&syntax.username.to_lowercase()),
		is_b2c: FREE_EMAIL_PROVIDERS_SET.contains(&syntax.domain.to_lowercase()),
		gravatar_url,
		haveibeenpwned,
	}
}
#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use super::*;
	use crate::{syntax::SyntaxDetails, EmailAddress};

	#[tokio::test]
	async fn test_check_misc() {
		let syntax = SyntaxDetails {
			address: Some(EmailAddress::from_str("test@gmail.com").unwrap()),
			is_valid_syntax: true,
			username: "test".to_string(),
			domain: "gmail.com".to_string(),
			normalized_email: None,
			suggestion: None,
		};

		let misc_details = check_misc(&syntax, true, None).await;

		assert!(!misc_details.is_disposable); // gmail.com is not in mailchecker
		assert!(misc_details.is_role_account); // test is in roles.txt
		assert!(misc_details.is_b2c); // gmail.com is in b2c.txt
	}
}
