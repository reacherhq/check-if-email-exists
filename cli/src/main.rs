// check-if-email-exists
// Copyright (C) 2018-2022 Reacher

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

use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailInputProxy};
use clap::Parser;
use once_cell::sync::Lazy;

/// CLI options of this binary.
#[derive(Parser, Debug)]
#[clap(version, about)]
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

	/// Username passed to the specified SOCKS5 proxy port to perform email verification.
	/// Only used when `--proxy-host` flag is set.
	#[clap(long, env)]
	pub proxy_username: Option<String>,

	/// Username passed to the specified SOCKS5 proxy port to perform email verification.
	/// Only used when `--proxy-host` flag is set.
	#[clap(long, env)]
	pub proxy_password: Option<String>,

	/// The port to use for the SMTP request.
	#[clap(long, env, default_value = "25")]
	pub smtp_port: u16,

	/// For Yahoo email addresses, use Yahoo's API instead of connecting
	/// directly to their SMTP servers.
	#[clap(long, env, default_value = "true", parse(try_from_str))]
	pub yahoo_use_api: bool,

	/// For Gmail email addresses, use Gmail's API instead of connecting
	/// directly to their SMTP servers.
	#[clap(long, env, default_value = "false", parse(try_from_str))]
	pub gmail_use_api: bool,

	/// For Microsoft 365 email addresses, use OneDrive's API instead of
	/// connecting directly to their SMTP servers.
	#[clap(long, env, default_value = "false", parse(try_from_str))]
	pub microsoft365_use_api: bool,

	/// Whether to check if a gravatar image is existing for the given email.
	#[clap(long, env, default_value = "false", parse(try_from_str))]
	pub check_gravatar: bool,

	/// The email to check.
	pub to_email: String,
}

/// Global config of this application.
pub(crate) static CONF: Lazy<Cli> = Lazy::new(Cli::parse);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	let to_email = &CONF.to_email;

	let mut input = CheckEmailInput::new(to_email.clone());
	input
		.set_from_email(CONF.from_email.clone())
		.set_hello_name(CONF.hello_name.clone())
		.set_smtp_port(CONF.smtp_port)
		.set_yahoo_use_api(CONF.yahoo_use_api)
		.set_gmail_use_api(CONF.gmail_use_api)
		.set_microsoft365_use_api(CONF.microsoft365_use_api)
		.set_check_gravatar(CONF.check_gravatar);
	if let Some(proxy_host) = &CONF.proxy_host {
		input.set_proxy(CheckEmailInputProxy {
			host: proxy_host.clone(),
			port: CONF.proxy_port,
			username: CONF.proxy_username.clone(),
			password: CONF.proxy_password.clone(),
		});
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

	Ok(())
}
