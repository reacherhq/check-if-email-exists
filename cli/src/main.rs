extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate check_if_email_exists_core;
extern crate lettre;

use check_if_email_exists_core::email_exists;
use clap::App;
use lettre::EmailAddress;
use std::process;
use std::str::FromStr;

/// Log error and exit immediately if there's one
macro_rules! try_or_exit (
    ($res: expr) => ({
		match $res {
			Ok(value) => value,
			Err(err) => {
				println!("{:?}", err);
				process::exit(1)
			}
		}
    })
);

fn main() {
	env_logger::init();

	// The YAML file is found relative to the current file, similar to how modules are found
	let yaml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let from_email = try_or_exit!(EmailAddress::from_str(
		matches.value_of("FROM_EMAIL").unwrap_or("user@example.org")
	));
	let to_email = try_or_exit!(EmailAddress::from_str(
		matches
			.value_of("TO_EMAIL")
			.expect("'TO_EMAIL' is required. qed.")
	));

	println!("This operation can take up to 1 minute, please be patient...");
	let exists = try_or_exit!(email_exists(&from_email, &to_email));

	println!("{}", exists)
}
