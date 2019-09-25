// check_if_email_exists
// Copyright (C) 2018-2019 Amaury Martiny

// check_if_email_exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check_if_email_exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check_if_email_exists.  If not, see <http://www.gnu.org/licenses/>.

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
	/// Can we send an email to this address?
	pub deliverable: bool,
	/// Is this email account's inbox full?
	pub full_inbox: bool,
	/// Does this domain have a catch-all email address?
	pub has_catch_all: bool,
}

/// Error occured connecting to this email server via SMTP
#[derive(Debug, Serialize)]
pub enum SmtpError {
	/// Skipped checking SMTP details
	Skipped,
	/// ISP is blocking SMTP ports
	BlockedByIsp,
	/// IO error when communicating with SMTP server
	#[serde(serialize_with = "ser_with_display")]
	Io(LettreSmtpError),
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

/// Check if `to_email` exists on host server with given port
fn email_deliverable(
	smtp_client: &mut InnerClient<NetworkStream>,
	to_email: &EmailAddress,
) -> Result<bool, LettreSmtpError> {
	// "RCPT TO: me@email.com"
	// FIXME Do not clone?
	let to_email = to_email.clone();
	match smtp_client.command(RcptCommand::new(to_email, vec![])) {
		Ok(response) => match response.first_line() {
			Some(message) => {
				if message.contains("2.1.5") {
					// 250 2.1.5 Recipient e-mail address ok.
					Ok(true)
				} else {
					Err(LettreSmtpError::Client("Can't find 2.1.5 in RCPT command"))
				}
			}
			None => Err(LettreSmtpError::Client("No response on RCPT command")),
		},
		Err(err) => {
			let err_string = err.to_string();
			// Don't return an error if the error contains anything about the
			// address being undeliverable
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
			{
				Ok(false)
			} else {
				Err(err)
			}
		}
	}
}

/// Verify the existence of a catch-all email
fn email_has_catch_all(
	smtp_client: &mut InnerClient<NetworkStream>,
	domain: &str,
) -> Result<bool, LettreSmtpError> {
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
}

/// Get all email details we can.
pub fn smtp_details(
	from_email: &EmailAddress,
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
) -> Result<SmtpDetails, LettreSmtpError> {
	let mut smtp_client = connect_to_host(from_email, host, port)?;

	let has_catch_all = email_has_catch_all(&mut smtp_client, domain).unwrap_or(false);
	let (deliverable, full_inbox) = match email_deliverable(&mut smtp_client, to_email) {
		Ok(exists) => (exists, false),
		Err(err) => {
			let err_string = err.to_string();
			// These messages mean that inbox is full, which also means that
			// email exists
			if err_string.contains("full")
				|| err_string.contains("insufficient")
				|| err_string.contains("over quota")
				|| err_string.contains("space")
			{
				(true, true)
			} else {
				debug!("Closing {}:{}, because of error '{}'.", host, port, err);
				return Err(err);
			}
		}
	};

	// Quit.
	smtp_client.close();

	Ok(SmtpDetails {
		deliverable,
		full_inbox,
		has_catch_all,
	})
}
