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

use fantoccini::{
	error::{CmdError, NewSessionError},
	Client, ClientBuilder,
};
use serde::Serialize;
use serde_json::Map;

use crate::util::ser_with_display::ser_with_display;

#[derive(Debug, Serialize)]
pub enum HeadlessError {
	#[serde(serialize_with = "ser_with_display")]
	Cmd(CmdError),
	#[serde(serialize_with = "ser_with_display")]
	NewSession(NewSessionError),
}

impl From<CmdError> for HeadlessError {
	fn from(e: CmdError) -> Self {
		Self::Cmd(e)
	}
}

impl From<NewSessionError> for HeadlessError {
	fn from(e: NewSessionError) -> Self {
		Self::NewSession(e)
	}
}

pub async fn create_headless_client(webdriver: &str) -> Result<Client, HeadlessError> {
	// Running in a Docker container, I run into the following error:
	// Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted
	// In searching around I found a few different workarounds:
	// - Enable namespaces: https://github.com/jessfraz/dockerfiles/issues/65#issuecomment-266532289
	// - Run it with a custom seccomp: https://github.com/jessfraz/dockerfiles/issues/65#issuecomment-217214671
	// - Run with --no-sandbox: https://github.com/karma-runner/karma-chrome-launcher/issues/125#issuecomment-312668593
	// For now I went with the --no-sandbox.
	//
	// TODO Look into security implications...
	let mut caps = Map::new();
	let opts = serde_json::json!({
		"args": ["--headless", "--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"],
	});
	caps.insert("goog:chromeOptions".to_string(), opts);

	// Connect to WebDriver instance that is listening on `webdriver`
	let c = ClientBuilder::native()
		.capabilities(caps)
		.connect(webdriver)
		.await?;

	Ok(c)
}
