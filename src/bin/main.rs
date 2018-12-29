#[macro_use]
extern crate clap;
extern crate check_if_email_exists;
extern crate lettre;
extern crate native_tls;
extern crate rayon;
extern crate trust_dns_resolver;

use check_if_email_exists::email_exists;
use clap::App;

fn main() {
	// The YAML file is found relative to the current file, similar to how modules are found
	let yaml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let from_email = matches.value_of("from").unwrap_or("user@example.org");
	// Calling .unwrap() is safe here because "TO" is required
	let to_email = matches.value_of("TO").unwrap();

	println!("{}", email_exists(from_email, to_email));
}
