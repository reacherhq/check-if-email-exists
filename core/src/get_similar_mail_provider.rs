use levenshtein::levenshtein;

use crate::syntax::SyntaxDetails;

const MAIL_PROVIDERS: &'static [&'static str] = &["gmail", "yahoo"];

pub fn get_similar_mail_provider(syntax: &mut SyntaxDetails) {
	let domain: &str = syntax.domain.as_ref();
	let mut domain_parts = domain.split(".");

	let provider: &str = domain_parts
		.next()
		.expect("We already checked that the syntax is valid")
		.into();
	let tld: &str = domain_parts
		.next()
		.expect("We already checked that the syntax is valid")
		.into();

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
