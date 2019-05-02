extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate check_if_email_exists;

use check_if_email_exists::email_exists;
use clap::App;

fn main() {
	env_logger::init();

	// The YAML file is found relative to the current file, similar to how modules are found
	let yaml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let from_email = matches.value_of("FROM_EMAIL").unwrap_or("user@example.org");
	// Calling .unwrap() is safe here because "TO" is required
	let to_email = matches.value_of("TO_EMAIL").unwrap();

	match email_exists(from_email, to_email) {
		Ok(value) => println!("{}", value),
		Err(err) => println!("{}", err),
	}
}
