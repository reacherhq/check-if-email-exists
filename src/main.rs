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

mod http;

use std::net::IpAddr;

use check_if_email_exists::{check_email, CheckEmailInput};
use clap::Clap;
use once_cell::sync::Lazy;

/// CLI options of this binary.
#[derive(Clap, Debug)]
#[clap(author, version, about)]
pub struct Cli {
	/// The email to use in the `MAIL FROM:` SMTP command.
	#[clap(long, env, default_value = "user@example.org")]
	pub from_email: String,

	/// The name to use in the `EHLO:` SMTP command.
	#[clap(long, env, default_value = "localhost")]
	pub hello_name: String,

	/// Use the specified SOCKS5 proxy host to perform email verification.
	#[clap(long, env)]
	pub proxy_host: Option<String>,

	/// Use the specified SOCKS5 proxy port to perform email verification.
	/// Only used when `--proxy-host` flag is set.
	#[clap(long, env, default_value = "1080")]
	pub proxy_port: u16,

	/// For Yahoo email addresses, use Yahoo's API instead of connecting
	/// directly to their SMTP servers.
	#[clap(long, env, default_value = "true", parse(try_from_str))]
	pub yahoo_use_api: bool,

	/// The email to check.
	pub to_email: Option<String>,

	/// Deprecated. Runs a HTTP server.
	/// This option will be removed in v0.9.
	#[clap(long)]
	#[deprecated(
		since = "0.8.24",
		note = "The HTTP server will be removed from the CLI, please use https://github.com/reacherhq/backend/ instead"
	)]
	pub http: bool,

	/// Deprecated. Sets the host IP address on which the HTTP server should bind.
	/// Only used when `--http` flag is on.
	/// This option will be removed in v0.9.
	#[clap(long, env = "HOST", default_value = "127.0.0.1")]
	#[deprecated(
		since = "0.8.24",
		note = "The HTTP server will be removed from the CLI, please use https://github.com/reacherhq/backend/ instead"
	)]
	pub http_host: IpAddr,

	/// Deprecated. Sets the port on which the HTTP server should bind.
	/// Only used when `--http` flag is on.
	/// If not set, then it will use $PORT, or default to 3000.
	///  This option will be removed in v0.9.
	#[clap(long, env = "PORT", default_value = "3000")]
	#[deprecated(
		since = "0.8.24",
		note = "The HTTP server will be removed from the CLI, please use https://github.com/reacherhq/backend/ instead"
	)]
	pub http_port: u16,
}

/// Global config of this application.
pub(crate) static CONF: Lazy<Cli> = Lazy::new(Cli::parse);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	if let Some(to_email) = &CONF.to_email {
		let mut input = CheckEmailInput::new(vec![to_email.clone()]);
		input
			.from_email(CONF.from_email.clone())
			.hello_name(CONF.hello_name.clone())
			.yahoo_use_api(CONF.yahoo_use_api);
		if let Some(proxy_host) = &CONF.proxy_host {
			input.proxy(proxy_host.clone(), CONF.proxy_port);
		}

		let result = check_email(&input).await;

		match serde_json::to_string_pretty(&result) {
			Ok(output) => {
				println!("{}", output);
			}
			Err(err) => {
				println!("{}", err);
			}
		};
	}

	// Run the web server if flag is on
	if CONF.http {
		http::run((CONF.http_host, CONF.http_port)).await?;
	}

	Ok(())
}
