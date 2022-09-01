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

use async_native_tls::TlsConnector;
use async_recursion::async_recursion;
use async_smtp::{
	smtp::{commands::*, extension::ClientId, ServerAddress, Socks5Config},
	ClientTlsParameters, EmailAddress, SmtpClient, SmtpTransport,
};
use async_std::future;
use rand::rngs::SmallRng;
use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use std::iter;
use std::str::FromStr;
use std::time::Duration;

use trust_dns_proto::rr::Name;

use super::parser;
use super::{SmtpDetails, SmtpError};
use crate::util::{constants::LOG_TARGET, input_output::CheckEmailInput};

/// Try to send an smtp command, close and return Err if fails.
macro_rules! try_smtp (
    ($res: expr, $client: ident, $to_email: expr, $host: expr, $port: expr) => ({
		if let Err(err) = $res {
			log::debug!(target: LOG_TARGET, "[email={}] Closing [host={}:{}], because of error '{:?}'.", $to_email, $host, $port, err);
			// Try to close the connection, but ignore if there's an error.
			let _ = $client.close().await;

			return Err(SmtpError::SmtpError(err));
		}
    })
);

/// Attempt to connect to host via SMTP, and return SMTP client on success.
async fn connect_to_host(
	host: &Name,
	port: u16,
	input: &CheckEmailInput,
) -> Result<SmtpTransport, SmtpError> {
	// hostname verification fails if it ends with '.', for example, using
	// SOCKS5 proxies we can `io: incomplete` error.
	let host = host.to_string();
	let host = host.trim_end_matches('.').to_string();

	let security = {
		let tls_params = ClientTlsParameters::new(host.clone(), TlsConnector::new().use_sni(true));

		input.smtp_security.to_client_security(tls_params)
	};

	let mut smtp_client = SmtpClient::with_security(
		ServerAddress {
			host: host.clone(),
			port,
		},
		security,
	)
	.hello_name(ClientId::Domain(input.hello_name.clone()))
	.timeout(Some(Duration::new(30, 0))); // Set timeout to 30s

	if let Some(proxy) = &input.proxy {
		let socks5_config = match (&proxy.username, &proxy.password) {
			(Some(username), Some(password)) => Socks5Config::new_with_user_pass(
				proxy.host.clone(),
				proxy.port,
				username.clone(),
				password.clone(),
			),
			_ => Socks5Config::new(proxy.host.clone(), proxy.port),
		};

		smtp_client = smtp_client.use_socks5(socks5_config);
	}

	let mut smtp_transport = smtp_client.into_transport();

	try_smtp!(
		smtp_transport.connect().await,
		smtp_transport,
		input.to_email,
		host,
		port
	);

	// "MAIL FROM: user@example.org"
	let from_email = EmailAddress::from_str(input.from_email.as_ref()).unwrap_or_else(|_| {
		log::warn!(
			"Inputted from_email \"{}\" is not a valid email, using \"user@example.org\" instead",
			input.from_email
		);
		EmailAddress::from_str("user@example.org").expect("This is a valid email. qed.")
	});
	try_smtp!(
		smtp_transport
			.command(MailCommand::new(Some(from_email), vec![],))
			.await,
		smtp_transport,
		input.to_email,
		host,
		port
	);

	Ok(smtp_transport)
}

/// Description of the deliverability information we can gather from
/// communicating with the SMTP server.
struct Deliverability {
	/// Is this email account's inbox full?
	has_full_inbox: bool,
	/// Can we send an email to this address?
	is_deliverable: bool,
	/// Is the email blocked or disabled by the provider?
	is_disabled: bool,
}

/// Check if `to_email` exists on host SMTP server. This is the core logic of
/// this tool.
async fn email_deliverable(
	smtp_transport: &mut SmtpTransport,
	to_email: &EmailAddress,
) -> Result<Deliverability, SmtpError> {
	// "RCPT TO: <target email>"
	// FIXME Do not clone `to_email`?
	match smtp_transport
		.command(RcptCommand::new(to_email.clone(), vec![]))
		.await
	{
		Ok(_) => {
			// According to RFC 5321, `RCPT TO` command succeeds with 250 and
			// 251 codes only (no 3xx codes at all):
			// https://tools.ietf.org/html/rfc5321#page-56
			//
			// Where the 251 code is used for forwarding, which is not our case,
			// because we always deliver to the SMTP server hosting the address
			// itself.
			//
			// So, if `response.is_positive()` (which is a condition for
			// returning `Ok` from the `command()` method above), then delivery
			// succeeds, accordingly to RFC 5321.
			Ok(Deliverability {
				has_full_inbox: false,
				is_deliverable: true, // response.is_positive()
				is_disabled: false,
			})
		}
		Err(err) => {
			// We cast to lowercase, because our matched strings below are all
			// lowercase.
			let err_string = err.to_string().to_lowercase();

			// Check if the email account has been disabled or blocked.
			if parser::is_disabled_account(err_string.as_str()) {
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: false,
					is_disabled: true,
				});
			}

			// Check if the email account has a full inbox.
			if parser::is_full_inbox(err_string.as_str()) {
				return Ok(Deliverability {
					has_full_inbox: true,
					is_deliverable: false,
					is_disabled: false,
				});
			}

			// Check error messages that say that user can actually receive
			// emails.
			// 4.2.1 The user you are trying to contact is receiving mail at a rate that
			if err_string
				.contains("the user you are trying to contact is receiving mail at a rate that")
			{
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: true,
					is_disabled: false,
				});
			}

			// Check that the mailbox doesn't exist.
			if parser::is_invalid(err_string.as_str(), to_email) {
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: false,
					is_disabled: false,
				});
			}

			// Return all unparsable errors,.
			Err(SmtpError::SmtpError(err))
		}
	}
}

/// Verify the existence of a catch-all on the domain.
async fn smtp_is_catch_all(
	smtp_transport: &mut SmtpTransport,
	domain: &str,
) -> Result<bool, SmtpError> {
	// Create a random 15-char alphanumerical string.
	let mut rng = SmallRng::from_entropy();
	let random_email: String = iter::repeat(())
		.map(|()| rng.sample(Alphanumeric))
		.map(char::from)
		.take(15)
		.collect();
	let random_email = EmailAddress::new(format!("{}@{}", random_email, domain));

	email_deliverable(
		smtp_transport,
		&random_email.expect("Email is correctly constructed. qed."),
	)
	.await
	.map(|deliverability| deliverability.is_deliverable)
}

async fn create_smtp_future(
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
) -> Result<(bool, Deliverability), SmtpError> {
	// FIXME If the SMTP is not connectable, we should actually return an
	// Ok(SmtpDetails { can_connect_smtp: false, ... }).
	let mut smtp_transport = connect_to_host(host, port, input).await?;

	let is_catch_all = smtp_is_catch_all(&mut smtp_transport, domain)
		.await
		.unwrap_or(false);
	let deliverability = if is_catch_all {
		Deliverability {
			has_full_inbox: false,
			is_deliverable: true,
			is_disabled: false,
		}
	} else {
		let mut result = email_deliverable(&mut smtp_transport, to_email).await;

		// Some SMTP servers automatically close the connection after an error,
		// so we should reconnect to perform a next command.
		//
		// Unfortunately `smtp_transport.is_connected()` doesn't report about this,
		// so we can only check for "io: incomplete" SMTP error being returned.
		// https://github.com/async-email/async-smtp/issues/37
		if let Err(e) = &result {
			if parser::is_err_io_errors(e) {
				log::debug!(
					target: LOG_TARGET,
					"Got `io: incomplete` error, reconnecting."
				);

				let _ = smtp_transport.close().await;
				smtp_transport = connect_to_host(host, port, input).await?;
				result = email_deliverable(&mut smtp_transport, to_email).await;
			}
		}

		result?
	};

	smtp_transport.close().await.map_err(SmtpError::SmtpError)?;

	Ok((is_catch_all, deliverability))
}

/// Get all email details we can from one single `EmailAddress`, without
/// retries.
async fn check_smtp_without_retry(
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
) -> Result<SmtpDetails, SmtpError> {
	let fut = create_smtp_future(to_email, host, port, domain, input);
	let (is_catch_all, deliverability) = if let Some(smtp_timeout) = input.smtp_timeout {
		future::timeout(smtp_timeout, fut).await??
	} else {
		fut.await?
	};

	Ok(SmtpDetails {
		can_connect_smtp: true,
		has_full_inbox: deliverability.has_full_inbox,
		is_catch_all,
		is_deliverable: deliverability.is_deliverable,
		is_disabled: deliverability.is_disabled,
	})
}

/// Get all email details we can from one single `EmailAddress`.
/// Retry the SMTP connection on error, in particular to avoid greylisting.
#[async_recursion]
pub async fn check_smtp_with_retry(
	to_email: &EmailAddress,
	host: &Name,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
	count: usize,
) -> Result<SmtpDetails, SmtpError> {
	log::debug!(
		target: LOG_TARGET,
		"[email={}] Check SMTP [attempt={}] on [host={}:{}]",
		input.to_email,
		input.retries - count + 1,
		host,
		port
	);

	let result = check_smtp_without_retry(to_email, host, port, domain, input).await;

	log::debug!(
		target: LOG_TARGET,
		"[email={}] Got result for [attempt={}] on [host={}:{}], [result={:?}]",
		input.to_email,
		input.retries - count + 1,
		host,
		port,
		result
	);

	match &result {
		// Don't retry if we used Hotmail or Yahoo API. This two options should
		// be non-callable, as this function only deals with actual SMTP
		// connection errors.
		#[cfg(feature = "headless")]
		Err(SmtpError::HotmailError(_)) => result,
		Err(SmtpError::YahooError(_)) => result,
		// Only retry if the SMTP error was unknown.
		Err(err) if err.get_description().is_none() => {
			if count <= 1 {
				result
			} else {
				log::debug!(
					target: LOG_TARGET,
					"[email={}] Potential greylisting detected, retrying.",
					input.to_email,
				);
				check_smtp_with_retry(to_email, host, port, domain, input, count - 1).await
			}
		}
		_ => result,
	}
}
