// check-if-email-exists
// Copyright (C) 2018-2023 Reacher

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

use crate::util::constants::LOG_TARGET;
use md5;
use md5::Digest;

const API_BASE_URL: &str = "https://www.gravatar.com/avatar/";

pub async fn check_gravatar(to_email: &str) -> Option<String> {
	let client = reqwest::Client::new();

	let mail_hash: Digest = md5::compute(to_email);

	let url = format!("{API_BASE_URL}{mail_hash:x}");

	log::debug!(
		target: LOG_TARGET,
		"[email={}] Request Gravatar API with url: {:?}",
		to_email,
		url
	);

	let response = client
		.get(&url)
		// This option is necessary to return a NotFound exception instead of the default gravatar
		// image if none for the given email is found.
		.query(&[("d", "404")])
		.send()
		.await;

	log::debug!(
		target: LOG_TARGET,
		"[email={}] Gravatar response: {:?}",
		to_email,
		response
	);

	let response = match response {
		Ok(response) => response,
		Err(_) => return None,
	};

	match response.status() {
		reqwest::StatusCode::OK => Some(url),
		_ => None,
	}
}
