use levenshtein::levenshtein;

use crate::syntax::SyntaxDetails;

const MAIL_PROVIDERS: &[&str] = &["gmail", "yahoo"];

pub fn get_similar_mail_provider(syntax: &mut SyntaxDetails) {
	let domain: &str = syntax.domain.as_ref();
	let mut domain_parts = domain.split('.');

	let provider = domain_parts
		.next()
		.expect("We already checked that the syntax is valid");
	let tld = domain_parts
		.next()
		.expect("We already checked that the syntax is valid");

	for possible_provider in MAIL_PROVIDERS {
		let distance = levenshtein(provider, possible_provider);

		if distance < 3 {
			// Return full address
			syntax.suggestion = Some(format!(
				"{}@{}.{}",
				syntax.username,
				String::from(*possible_provider),
				tld
			));
			break;
		}
	}
}
