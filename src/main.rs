// check-if-email-exists
// Copyright (C) 2018-2019 Amaury Martiny

// check-if-email-exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check-if-email-exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check-if-email-exists.  If not, see <http://www.gnu.org/licenses/>.

extern crate clap;
extern crate env_logger;
extern crate serde;

use check_if_email_exists::email_exists;
use clap::App;
use serde_json;

fn main() -> serde_json::Result<()> {
	env_logger::init();

	// The YAML file is found relative to the current file, similar to how modules are found
	let yaml = clap::load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let from_email = matches
		.value_of("FROM_EMAIL")
		.expect("FROM_EMAIL has a default value. qed.");
	let to_email = matches
		.value_of("TO_EMAIL")
		.expect("TO_EMAIL is required. qed.");

	let output = serde_json::to_string_pretty(&email_exists(&to_email, &from_email))?;
	println!("{}", output);

	Ok(())
}
