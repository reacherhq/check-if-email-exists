pub fn normalize_email(email_address: &str) -> String {
	let (username, domain) = email_address
		.rsplit_once('@')
		.expect("Email syntax already verified.");

	match domain {
		"gmail.com" | "googlemail.com" => normalize_gmail(username),
		_ => email_address.into(),
	}
}

/// Normalize a Gmail address.
///
/// See Gmail username
/// [restrictions](https://support.google.com/mail/answer/9211434?hl=en-GB).
///
/// - removes
///   [sub-addresses](https://support.google.com/a/users/answer/9282734?hl=en#zippy=%2Clearn-how)
///   (i.e. parts after a `+` character.)
/// - removes [dots](https://support.google.com/mail/answer/7436150).
/// - converts to lower-case.
/// - [replaces](https://support.google.com/mail/answer/10313?hl=en-GB#zippy=%2Cgetting-messages-sent-to-an-googlemailcom-address)
///   `googlemail.com` with `gmail.com`.
fn normalize_gmail(username: &str) -> String {
	let username = match username.split_once('+') {
		Some((username, _)) => username,
		_ => username,
	}
	.chars()
	.filter_map(|c| match c.to_ascii_lowercase() {
		'.' => None,
		lower => Some(lower),
	})
	.collect::<String>();

	format!("{}@gmail.com", username)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_gmail_removes_periods() {
		assert_eq!(normalize_email("a.b.c@gmail.com"), "abc@gmail.com");
	}

	#[test]
	fn test_gmail_removes_subaddress() {
		assert_eq!(normalize_email("abc+123@gmail.com"), "abc@gmail.com");
	}

	#[test]
	fn test_gmail_uses_gmail_com() {
		assert_eq!(normalize_email("abc@googlemail.com"), "abc@gmail.com");
	}

	#[test]
	fn test_gmail() {
		assert_eq!(normalize_email("ABC+123@googlemail.com"), "abc@gmail.com");
	}
}
