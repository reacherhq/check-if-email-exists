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

use crate::{util::ser_with_display::ser_with_display, WebdriverConfig};
use fantoccini::{
	error::{CmdError, NewSessionError},
	Client, ClientBuilder,
};
use serde::Serialize;
use serde_json::Map;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum HeadlessError {
	#[serde(serialize_with = "ser_with_display")]
	#[error("Fantoccini error: {0}")]
	Fantoccini(#[from] std::io::Error),
	#[serde(serialize_with = "ser_with_display")]
	#[error("Cmd error: {0}")]
	Cmd(#[from] CmdError),
	#[serde(serialize_with = "ser_with_display")]
	#[error("New session error: {0}")]
	NewSession(#[from] NewSessionError),
}

pub async fn create_headless_client(
	webdriver: &str,
	webdriver_config: &WebdriverConfig,
) -> Result<Client, HeadlessError> {
	let mut opts = serde_json::json!({
		"args": [
			"--headless=new", "--disable-gpu", "--disable-dev-shm-usage",
			// Running in a Docker container, I run into the following error:
			// Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted
			// In searching around I found a few different workarounds:
			// - Enable namespaces: https://github.com/jessfraz/dockerfiles/issues/65#issuecomment-266532289
			// - Run it with a custom seccomp: https://github.com/jessfraz/dockerfiles/issues/65#issuecomment-217214671
			// - Run with --no-sandbox: https://github.com/karma-runner/karma-chrome-launcher/issues/125#issuecomment-312668593
			// For now I went with the --no-sandbox.
			//
			// TODO Look into security implications...
			"--no-sandbox",
			// From https://github.com/chromium-for-lambda/chromium-binaries/tree/b23e11c2f2859b177fd08fe50a0826c17652d846?tab=readme-ov-file#installation-via-a-lambda-layer
			"--use-gl=angle", "--use-angle=swiftshader", "--single-process", "--no-zygote",
			// Disable anything that might consume memory
			"--window-size=800x600",
			"--disable-extensions",
			"--disable-software-rasterizer",
			"--disable-dev-shm-usage",
			"--disable-background-networking",
			"--js-flags=\"--max-old-space-size=256\"",
		]
	});

	if let Some(binary) = &webdriver_config.binary {
		opts["binary"] = serde_json::json!(binary);
	}

	let mut caps = Map::new();
	caps.insert("goog:chromeOptions".to_string(), opts);

	// Connect to WebDriver instance that is listening on `webdriver`
	let c = ClientBuilder::rustls()?
		.capabilities(caps)
		.connect(webdriver)
		.await?;

	Ok(c)
}
