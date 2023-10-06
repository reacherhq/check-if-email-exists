#[cfg(feature = "headless")]
pub mod hotmail;
pub mod microsoft365;

/// Check if a MX host is from outlook (includes @hotmail.*, @outlook.* and
/// all Microsoft 365 addresses).
pub fn is_outlook(host: &str) -> bool {
	host.to_lowercase()
		.ends_with(".mail.protection.outlook.com.")
}

/// Check if a MX host is an @hotmail.* or @outlook.* email.
///
/// After some testing, I got:
/// - *@outlook.com -> `outlook-com.olc.protection.outlook.com.`
/// - *@outlook.fr -> `eur.olc.protection.outlook.com.`
/// - *@hotmail.com -> `hotmail-com.olc.protection.outlook.com.`
/// - *@hotmail.fr -> `eur.olc.protection.outlook.com.`
/// - *@hotmail.nl -> `eur.olc.protection.outlook.com.`
///
/// So it seems that outlook/hotmail addresses end with `olc.protection.outlook.com.`
pub fn is_hotmail(host: &str) -> bool {
	host.to_lowercase()
		.ends_with(".mail.protection.outlook.com.")
}
