#[cfg(feature = "headless")]
pub mod hotmail;
pub mod microsoft365;

/// Check if a MX host is from outlook.
pub fn is_outlook(host: &str) -> bool {
	host.to_lowercase()
		.ends_with(".mail.protection.outlook.com.")
}
