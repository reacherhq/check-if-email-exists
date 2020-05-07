// check-if-email-exists
// Copyright (C) 2018-2020 Amaury Martiny

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

/// E2E tests

#[cfg(test)]
mod tests {
	use check_if_email_exists::{check_emails, CheckEmailInput};
	use serde_json;
	use std::fs;
	use tokio::runtime::Runtime;

	/// Function to test all fixtures from a folder.
	fn test_all_from_folder(folder: &str) {
		let mut runtime = Runtime::new().unwrap();
		let paths = fs::read_dir(folder).unwrap();

		// For every fixture file, we compare:
		// - the result of `check_emails` with the email in the filename
		// - the contents of the file
		for path in paths {
			let path = path.unwrap().path();
			let file = fs::File::open(path.clone()).unwrap();
			let filename = path.file_stem().unwrap().to_str().unwrap();
			let json: serde_json::Value = serde_json::from_reader(file).unwrap();

			println!("Check {}", filename);
			let mut input = CheckEmailInput::new(vec![filename.into()]);
			input.proxy("127.0.0.1".into(), 9050);
			let result = runtime.block_on(check_emails(&input));

			// Uncomment to see the JSON result of `check_emails`.
			println!("{}", serde_json::to_string(&result[0]).unwrap());

			assert_eq!(json, serde_json::to_value(&result[0]).unwrap());
		}
	}

	#[test]
	fn should_pass_fixtures() {
		test_all_from_folder("./src/fixtures");
	}

	#[test]
	fn should_pass_sensitive_fixtures() {
		// These fixtures contain real-file emails, they are not committed to
		// git.
		test_all_from_folder("./src/sensitive_fixtures");
	}
}
