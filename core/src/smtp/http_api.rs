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

use crate::util::{constants::LOG_TARGET, input_output::CheckEmailInput};
use reqwest::Error as ReqwestError;

/// Helper function to create a reqwest client, with optional proxy.
pub fn create_client(
	input: &CheckEmailInput,
	api_name: &str,
) -> Result<reqwest::Client, ReqwestError> {
	if let Some(proxy) = &input.proxy {
		log::debug!(
			target: LOG_TARGET,
			"[email={}] Using proxy socks5://{}:{} for {} API",
			input.to_email,
			proxy.host,
			proxy.port,
			api_name,
		);

		let proxy = if proxy.username.is_some() && proxy.password.is_some() {
			reqwest::Proxy::all(format!(
				"socks5://{}:{}@{}:{}",
				proxy.username.as_ref().unwrap(),
				proxy.password.as_ref().unwrap(),
				proxy.host,
				proxy.port
			))?
		} else {
			reqwest::Proxy::all(format!("socks5://{}:{}", proxy.host, proxy.port))?
		};

		reqwest::Client::builder().proxy(proxy).build()
	} else {
		Ok(reqwest::Client::new())
	}
}
