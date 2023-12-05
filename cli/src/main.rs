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

use check_if_email_exists::{
	check_email, CheckEmailInput, CheckEmailInputProxy, GmailVerifMethod, HotmailVerifMethod,
	YahooVerifMethod,
};
use clap::Parser;
use once_cell::sync::Lazy;

/// CLI options of this binary.
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Cli {
	/// The email to use in the `MAIL FROM:` SMTP command.
	#[clap(long, env, default_value = "reacher.email@gmail.com")]
	pub from_email: String,

	/// The name to use in the `EHLO:` SMTP command.
	#[clap(long, env, default_value = "gmail.com")]
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

	/// Select how to verify Yahoo email addresses: Api, Headless or Smtp.
	#[clap(long, env, default_value = "Headless", parse(try_from_str))]
	pub yahoo_verif_method: YahooVerifMethod,

	/// Select how to verify Gmail email addresses: Api or Smtp.
	#[clap(long, env, default_value = "Smtp", parse(try_from_str))]
	pub gmail_verif_method: GmailVerifMethod,

	/// Select how to verify Hotmail email addresses: Api, Headless or Smtp.
	#[clap(long, env, default_value = "Headless", parse(try_from_str))]
	pub hotmail_verif_method: HotmailVerifMethod,

	/// Whether to check if a gravatar image is existing for the given email.
	#[clap(long, env, default_value = "false", parse(try_from_str))]
	pub check_gravatar: bool,

	/// HaveIBeenPnwed API key, ignore if not provided.
	#[clap(long, env, parse(try_from_str))]
	pub haveibeenpwned_api_key: Option<String>,

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
		.set_yahoo_verif_method(CONF.yahoo_verif_method)
		.set_gmail_verif_method(CONF.gmail_verif_method)
		.set_hotmail_verif_method(CONF.hotmail_verif_method)
		.set_check_gravatar(CONF.check_gravatar)
		.set_haveibeenpwned_api_key(CONF.haveibeenpwned_api_key.clone());

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
			println!("{output}");
		}
		Err(err) => {
			println!("{err}");
		}
	};

	Ok(())
}
