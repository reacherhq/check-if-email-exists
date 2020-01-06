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
	use check_if_email_exists::email_exists;
	use futures::executor::block_on;

	#[test]
	fn should_output_error_for_invalid_email() {
		let result = block_on(email_exists("foo", "user@example.org"));
		assert_eq!(
			serde_json::to_string(&result).unwrap(),
			"{\"input\":\"foo\",\"misc\":{\"error\":{\"type\":\"Skipped\"}},\"mx\":{\"error\":{\"type\":\"Skipped\"}},\"smtp\":{\"error\":{\"type\":\"Skipped\"}},\"syntax\":{\"error\":{\"type\":\"SyntaxError\",\"message\":\"invalid email address\"}}}"
		);
	}

	#[test]
	fn should_output_error_for_invalid_mx() {
		let result = block_on(email_exists("foo@bar.baz", "user@example.org"));

		assert_eq!(
			serde_json::to_string(&result).unwrap(),
			"{\"input\":\"foo@bar.baz\",\"misc\":{\"error\":{\"type\":\"Skipped\"}},\"mx\":{\"error\":{\"type\":\"ResolveError\",\"message\":\"no record found for name: bar.baz type: MX class: IN\"}},\"smtp\":{\"error\":{\"type\":\"Skipped\"}},\"syntax\":{\"address\":\"foo@bar.baz\",\"domain\":\"bar.baz\",\"username\":\"foo\",\"valid_format\":true}}"
		);
	}
}
