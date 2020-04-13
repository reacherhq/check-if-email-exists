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

use crate::util::ser_with_display::ser_with_display;
use async_smtp::{
	smtp::{
		client::net::NetworkStream, commands::*, error::Error as AsyncSmtpError,
		extension::ClientId,
	},
	ClientSecurity, EmailAddress, SmtpClient, SmtpTransport,
};
use fast_socks5::{
	client::{Config, Socks5Stream},
	Result, SocksError,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use std::time::Duration;
use trust_dns_resolver::Name;

use super::util::email_input::EmailInputProxy;

/// Details that we gathered from connecting to this email via SMTP
#[derive(Debug, Serialize)]
pub struct SmtpDetails {
	/// Is this email account's inbox full?
	pub has_full_inbox: bool,
	/// Does this domain have a catch-all email address?
	pub is_catch_all: bool,
	/// Can we send an email to this address?
	pub is_deliverable: bool,
	/// Is the email blocked or disabled by the provider?
	pub is_disabled: bool,
}

/// Error occured connecting to this email server via SMTP
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum SmtpError {
	/// Skipped checking SMTP details
	Skipped,
	/// Error if we're using a SOCKS5 proxy
	#[serde(serialize_with = "ser_with_display")]
	SocksError(SocksError),
	/// Error when communicating with SMTP server
	#[serde(serialize_with = "ser_with_display")]
	SmtpError(AsyncSmtpError),
}

impl From<AsyncSmtpError> for SmtpError {
	fn from(error: AsyncSmtpError) -> Self {
		SmtpError::SmtpError(error)
	}
}

impl From<SocksError> for SmtpError {
	fn from(error: SocksError) -> Self {
		SmtpError::SocksError(error)
	}
}

/// Try to send an smtp command, close and return Err if fails.
macro_rules! try_smtp (
    ($res: expr, $client: ident, $host: expr, $port: expr) => ({
		if let Err(err) = $res {
			debug!("Closing {}:{}, because of error '{}'.", $host, $port, err);
			$client.close().await?;

			return Err(err.into());
		}
    })
);

/// Attempt to connect to host via SMTP, and return SMTP client on success
async fn connect_to_host(
	from_email: &EmailAddress,
	host: &Name,
	port: u16,
	hello_name: &str,
	proxy: &Option<EmailInputProxy>,
) -> Result<SmtpTransport, SmtpError> {
	let mut smtp_client =
		SmtpClient::with_security((host.to_utf8().as_str(), port), ClientSecurity::None)
			.await?
			.hello_name(ClientId::Domain(hello_name.into()))
			.timeout(Some(Duration::new(30, 0))) // Set timeout to 30s
			.into_transport();

	// Connect to the host. If the proxy argument is set, use it.
	debug!("Connecting to {}:{}", host, port);
	if let Some(proxy) = proxy {
		let stream = Socks5Stream::connect(
			(proxy.host.as_ref(), proxy.port),
			host.to_utf8(),
			port,
			Config::default(),
		)
		.await?;

		try_smtp!(
			smtp_client
				.connect_with_stream(NetworkStream::Socks5Stream(stream))
				.await,
			smtp_client,
			host,
			port
		);
	} else {
		try_smtp!(smtp_client.connect().await, smtp_client, host, port);
	}

	// "MAIL FROM: user@example.org"
	// FIXME Do not clone?
	let from_email = from_email.clone();
	try_smtp!(
		smtp_client
			.command(MailCommand::new(Some(from_email), vec![],))
			.await,
		smtp_client,
		host,
		port
	);

	Ok(smtp_client)
}

struct Deliverability {
	/// Is this email account's inbox full?
	pub has_full_inbox: bool,
	/// Can we send an email to this address?
	is_deliverable: bool,
	/// Is the email blocked or disabled by the provider?
	is_disabled: bool,
}

/// Check if `to_email` exists on host server with given port
async fn email_deliverable(
	smtp_client: &mut SmtpTransport,
	to_email: &EmailAddress,
) -> Result<Deliverability, SmtpError> {
	// "RCPT TO: me@email.com"
	// FIXME Do not clone?
	let to_email = to_email.clone();
	match smtp_client
		.command(RcptCommand::new(to_email, vec![]))
		.await
	{
		Ok(response) => match response.first_line() {
			Some(message) => {
				if message.contains("2.1.5") {
					// 250 2.1.5 Recipient e-mail address ok.
					Ok(Deliverability {
						has_full_inbox: false,
						is_deliverable: true,
						is_disabled: false,
					})
				} else {
					Err(SmtpError::SmtpError(AsyncSmtpError::Client(
						"Can't find 2.1.5 in RCPT command",
					)))
				}
			}
			None => Err(SmtpError::SmtpError(AsyncSmtpError::Client(
				"No response on RCPT command",
			))),
		},
		Err(err) => {
			let err_string = err.to_string();
			// Don't return an error if the error contains anything about the
			// address being undeliverable

			// Check if the email account has been disabled or blocked
			// e.g. "The email account that you tried to reach is disabled. Learn more at https://support.google.com/mail/?p=DisabledUser"
			if err_string.contains("disabled") || err_string.contains("blocked") {
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: false,
					is_disabled: true,
				});
			}

			// Check if the email account has a full inbox
			if err_string.contains("full")
				|| err_string.contains("insufficient")
				|| err_string.contains("over quota")
				|| err_string.contains("space")
			{
				return Ok(Deliverability {
					has_full_inbox: true,
					is_deliverable: false,
					is_disabled: false,
				});
			}

			// These are the possible error messages when email account doesn't exist
			if err_string.contains("address rejected")
				|| err_string.contains("does not exist")
				|| err_string.contains("invalid address")
				|| err_string.contains("may not exist")
				|| err_string.contains("no mailbox")
				|| err_string.contains("recipient invalid")
				|| err_string.contains("recipient rejected")
				|| err_string.contains("undeliverable")
				|| err_string.contains("user unknown")
				|| err_string.contains("user not found")
				|| err_string.contains("disabled")
			{
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: false,
					is_disabled: false,
				});
			}

			Err(SmtpError::SmtpError(err))
		}
	}
}

/// Verify the existence of a catch-all email
async fn email_has_catch_all(
	smtp_client: &mut SmtpTransport,
	domain: &str,
) -> Result<bool, SmtpError> {
	// Create a random 15-char alphanumerical string
	let random_email = rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(15)
		.collect::<String>();
	let random_email = EmailAddress::new(format!("{}@{}", random_email, domain));

	email_deliverable(
		smtp_client,
		&random_email.expect("Email is correctly constructed. qed."),
	)
	.await
	.map(|deliverability| deliverability.is_deliverable)
}

/// Get all email details we can.
pub async fn smtp_details(
	from_email: &EmailAddress,
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
	hello_name: &str,
	proxy: &Option<EmailInputProxy>,
) -> Result<SmtpDetails, SmtpError> {
	let mut smtp_client = connect_to_host(from_email, host, port, hello_name, proxy).await?;

	let is_catch_all = email_has_catch_all(&mut smtp_client, domain)
		.await
		.unwrap_or(false);
	let deliverability = email_deliverable(&mut smtp_client, to_email).await?;

	smtp_client.close().await?;

	Ok(SmtpDetails {
		has_full_inbox: deliverability.has_full_inbox,
		is_catch_all,
		is_deliverable: deliverability.is_deliverable,
		is_disabled: deliverability.is_disabled,
	})
}
