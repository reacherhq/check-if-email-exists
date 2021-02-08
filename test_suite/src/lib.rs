// check-if-email-exists
// Copyright (C) 2018-2021 Reacher

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! E2E tests

#[cfg(test)]
mod tests {
	use check_if_email_exists::{check_email, CheckEmailInput};
	use serde_json;
	use std::fs;
	use tokio::runtime::Runtime;

	/// Function to test all fixtures from a folder.
	fn test_all_from_folder(folder: &str) {
		let mut runtime = Runtime::new().unwrap();
		let paths = fs::read_dir(folder).unwrap();

		// For every fixture file, we compare:
		// - the result of `check_email` with the email in the filename
		// - the contents of the file
		for path in paths {
			let path = path.unwrap().path();
			let file = fs::File::open(path.clone()).unwrap();
			let filename = path.file_stem().unwrap().to_str().unwrap();
			if filename.starts_with(".") {
				// Don't process hidden files.
				continue;
			}
			let expected: serde_json::Value = serde_json::from_reader(file).unwrap();

			println!("Check {}", filename);
			let input = CheckEmailInput::new(vec![filename.into()]);
			let result = runtime.block_on(check_email(&input));
			let actual = serde_json::to_value(&result[0]).unwrap();

			// Uncomment to see the JSON result of `check_email`.
			// println!("{}", actual);

			// For the input,misc,smtp,syntax fields, we match exact JSON.
			assert_eq!(expected.get("input"), actual.get("input"),);
			assert_eq!(expected.get("is_reachable"), actual.get("is_reachable"),);
			assert_eq!(expected.get("misc"), actual.get("misc"),);
			assert_eq!(expected.get("smtp"), actual.get("smtp"),);
			assert_eq!(expected.get("syntax"), actual.get("syntax"),);

			// For the mx field, the `records` field array can contain elements
			// in different order. We only check length in that case
			if expected.get("mx").unwrap().get("error").is_some() {
				assert_eq!(expected.get("mx"), actual.get("mx"));
			} else {
				assert_eq!(
					expected.get("mx").unwrap().get("accepts_mail"),
					actual.get("mx").unwrap().get("accepts_mail")
				);

				assert_eq!(
					expected
						.get("mx")
						.unwrap()
						.get("records")
						.unwrap()
						.as_array()
						.unwrap()
						.len(),
					actual
						.get("mx")
						.unwrap()
						.get("records")
						.unwrap()
						.as_array()
						.unwrap()
						.len()
				);
			}
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
