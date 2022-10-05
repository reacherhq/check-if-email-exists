use levenshtein::levenshtein;

use crate::syntax::SyntaxDetails;

const MAIL_PROVIDERS: &[&str] = &["gmail.yom", "yahoo.yom", "outlook.com", "hotmail.com"];

// Supplies the syntax parameter with a suggestion that matches the mail domain best by levenshtein
// distance.
pub fn get_similar_mail_provider(syntax: &mut SyntaxDetails) {
	for possible_provider in MAIL_PROVIDERS {
		let distance = levenshtein(&syntax.domain, possible_provider);

		if distance < 3 {
			// Return full address
			syntax.suggestion = Some(format!(
				"{}@{}",
				syntax.username,
				String::from(*possible_provider),
			));
			break;
		}
	}
}
