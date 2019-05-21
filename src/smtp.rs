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

use lettre::smtp::client::net::NetworkStream;
use lettre::smtp::client::InnerClient;
use lettre::smtp::commands::*;
use lettre::smtp::error::Error;
use lettre::smtp::extension::ClientId;
use lettre::EmailAddress;
use std::time::Duration;
use trust_dns_resolver::Name;

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

/// Check if `to_email` exists on host server with given port
pub fn email_exists_on_host(
	from_email: &EmailAddress,
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
) -> Result<bool, Error> {
	debug!("Connecting to {}:{}", host, port);
	let mut smtp_client: InnerClient<NetworkStream> = InnerClient::new();
	let timeout = Some(Duration::new(3, 0)); // Set timeout to 3s

	// Set timeout.
	if let Err(err) = smtp_client.set_timeout(timeout) {
		debug!("Closing {}:{}, because of error '{}'.", host, port, err);
		smtp_client.close();
		return Err(Error::from(err));
	}

	// Connect to the host.
	try_smtp!(
		smtp_client.connect(&(host.to_utf8().as_str(), port), timeout, None),
		smtp_client,
		host,
		port
	);

	// Send ehlo.
	try_smtp!(
		smtp_client.command(EhloCommand::new(ClientId::new("localhost".to_string()))),
		smtp_client,
		host,
		port
	);

	// Send from.
	// FIXME Do not clone?
	let from_email_clone = from_email.clone();
	try_smtp!(
		smtp_client.command(MailCommand::new(Some(from_email_clone), vec![],)),
		smtp_client,
		host,
		port
	);

	// Send to.
	// FIXME Do not clone?
	let to_email_clone = to_email.clone();
	let result = match smtp_client.command(RcptCommand::new(to_email_clone, vec![])) {
		Ok(response) => match response.first_line() {
			Some(message) => {
				// 250 2.1.5 Recipient e-mail address ok.
				if message.contains("2.1.5") {
					Ok(true)
				} else {
					Err(Error::Client("Can't find 2.1.5 in RCPT command"))
				}
			}
			None => Err(Error::Client("No response on RCPT command")),
		},
		Err(err) => {
			// 550 5.1.1 Mailbox does not exist.
			if err.to_string().contains("5.1.1") {
				Ok(false)
			} else {
				Err(err)
			}
		}
	};

	// Quit.
	smtp_client.close();

	match result {
		Ok(val) => debug!("Checked email on {}:{}, exists={}.", host, port, val),
		Err(_) => debug!("Cannot check email on {}:{}.", host, port),
	};
	result
}
