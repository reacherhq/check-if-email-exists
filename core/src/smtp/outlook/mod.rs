#[cfg(feature = "headless")]
pub mod headless;
pub mod microsoft365;

/// Check if a MX host is from outlook (includes @hotmail.*, @outlook.* and
/// all Microsoft 365 addresses).
///
/// After some testing I got:
/// After some testing, I got:
/// - *@outlook.com -> `outlook-com.olc.protection.outlook.com.`
/// - *@outlook.fr -> `eur.olc.protection.outlook.com.`
/// - *@hotmail.com -> `hotmail-com.olc.protection.outlook.com.`
/// - *@hotmail.fr -> `eur.olc.protection.outlook.com.`
/// - *@hotmail.nl -> `eur.olc.protection.outlook.com.`
///
/// But Microsoft 365 addresses end with "mail.protection.outlook.com."
///
/// TL;DR:
/// - @outlook.* and @hotmail.* -> end with ".olc.protection.outlook.com."
/// - Microsoft 365 emails -> end with ".mail.protection.outlook.com."
pub fn is_hotmail(host: &str) -> bool {
	host.to_lowercase().ends_with(".protection.outlook.com.")
}

/// Check if an address is a Microsoft365 email address.
pub fn is_microsoft365(host: &str) -> bool {
	is_hotmail(host) && !host.ends_with(".olc.protection.outlook.com.")
}
