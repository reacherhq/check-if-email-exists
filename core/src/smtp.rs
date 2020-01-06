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

use crate::util::ser_with_display;
use lettre::smtp::client::net::NetworkStream;
use lettre::smtp::client::InnerClient;
use lettre::smtp::commands::*;
use lettre::smtp::error::Error as LettreSmtpError;
use lettre::smtp::extension::ClientId;
use lettre::EmailAddress;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Serialize;
use std::time::Duration;
use trust_dns_resolver::Name;

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
	/// Error when communicating with SMTP server
	#[serde(serialize_with = "ser_with_display")]
	LettreError(LettreSmtpError),
}

/// Try to send an smtp command, close and return Err if fails.
macro_rules! try_smtp (
    ($res: expr, $client: ident, $host: expr, $port: expr) => ({
		if let Err(err) = $res {
			debug!("Closing {}:{}, because of error '{}'.", $host, $port, err);
			$client.close();
			return Err(err);
		}
    })
);

/// Attempt to connect to host via SMTP, and return SMTP client on success
fn connect_to_host(
	from_email: &EmailAddress,
	host: &Name,
	port: u16,
) -> Result<InnerClient<NetworkStream>, LettreSmtpError> {
	debug!("Connecting to {}:{}", host, port);
	let mut smtp_client: InnerClient<NetworkStream> = InnerClient::new();
	let timeout = Some(Duration::new(3, 0)); // Set timeout to 3s

	// Set timeout.
	if let Err(err) = smtp_client.set_timeout(timeout) {
		debug!("Closing {}:{}, because of error '{}'.", host, port, err);
		smtp_client.close();
		return Err(LettreSmtpError::from(err));
	}

	// Connect to the host.
	try_smtp!(
		smtp_client.connect(&(host.to_utf8().as_str(), port), None),
		smtp_client,
		host,
		port
	);

	// "EHLO localhost"
	try_smtp!(
		smtp_client.command(EhloCommand::new(ClientId::new("localhost".to_string()))),
		smtp_client,
		host,
		port
	);

	// "MAIL FROM: user@example.org"
	// FIXME Do not clone?
	let from_email = from_email.clone();
	try_smtp!(
		smtp_client.command(MailCommand::new(Some(from_email), vec![],)),
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
fn email_deliverable(
	smtp_client: &mut InnerClient<NetworkStream>,
	to_email: &EmailAddress,
) -> Result<Deliverability, SmtpError> {
	// "RCPT TO: me@email.com"
	// FIXME Do not clone?
	let to_email = to_email.clone();
	match smtp_client.command(RcptCommand::new(to_email, vec![])) {
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
					Err(SmtpError::LettreError(LettreSmtpError::Client(
						"Can't find 2.1.5 in RCPT command",
					)))
				}
			}
			None => Err(SmtpError::LettreError(LettreSmtpError::Client(
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

			Err(SmtpError::LettreError(err))
		}
	}
}

/// Verify the existence of a catch-all email
fn email_has_catch_all(
	smtp_client: &mut InnerClient<NetworkStream>,
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
	.map(|deliverability| deliverability.is_deliverable)
}

/// Get all email details we can.
pub async fn smtp_details(
	from_email: &EmailAddress,
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
) -> Result<SmtpDetails, SmtpError> {
	let mut smtp_client = match connect_to_host(from_email, host, port) {
		Ok(client) => client,
		Err(err) => return Err(SmtpError::LettreError(err)),
	};

	let is_catch_all = email_has_catch_all(&mut smtp_client, domain).unwrap_or(false);
	let deliverability = email_deliverable(&mut smtp_client, to_email)?;

	smtp_client.close();

	Ok(SmtpDetails {
		has_full_inbox: deliverability.has_full_inbox,
		is_catch_all,
		is_deliverable: deliverability.is_deliverable,
		is_disabled: deliverability.is_disabled,
	})
}
