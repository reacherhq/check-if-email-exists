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

use async_recursion::async_recursion;
use async_smtp::commands::{MailCommand, RcptCommand};
use async_smtp::extension::ClientId;
use async_smtp::{SmtpClient, SmtpTransport};
use fast_socks5::client::Config;
use fast_socks5::{client::Socks5Stream, Result};
use rand::rngs::SmallRng;
use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use std::iter;
use std::str::FromStr;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream};
use tokio::net::TcpStream;

use super::parser;
use super::{SmtpDetails, SmtpError};
use crate::{
	rules::{has_rule, Rule},
	util::input_output::CheckEmailInput,
};
use crate::{EmailAddress, LOG_TARGET};
use tracing::{debug, warn};

// Define a new trait that combines AsyncRead, AsyncWrite, and Unpin
trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncReadWrite for T {}

/// Try to send an smtp command, close and return Err if fails.
macro_rules! try_smtp (
    ($res: expr, $client: ident, $to_email: expr, $host: expr, $port: expr) => ({
		if let Err(err) = $res {
			debug!(
				target: LOG_TARGET,
				"[email={}] Closing [host={}:{}], because of error '{:?}'.",
				$to_email, $host, $port,err,
			);
			// Try to close the connection, but ignore if there's an error.
			let _ = $client.quit().await;

			return Err(SmtpError::AsyncSmtpError(err));
		}
    })
);

/// Connect to an SMTP host and return the configured client transport.
async fn connect_to_smtp_host(
	host: &str,
	port: u16,
	input: &CheckEmailInput,
) -> Result<SmtpTransport<BufStream<Box<dyn AsyncReadWrite>>>, SmtpError> {
	// hostname verification fails if it ends with '.', for example, using
	// SOCKS5 proxies we can `io: incomplete` error.
	let clean_host = host.trim_end_matches('.').to_string();
	let smtp_client = SmtpClient::new().hello_name(ClientId::Domain(input.hello_name.clone()));

	let stream: BufStream<Box<dyn AsyncReadWrite>> = match &input.proxy {
		Some(proxy) => {
			let socks_stream =
				if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
					Socks5Stream::connect_with_password(
						(proxy.host.as_ref(), proxy.port),
						clean_host.clone(),
						port,
						username.clone(),
						password.clone(),
						Config::default(),
					)
					.await?
				} else {
					Socks5Stream::connect(
						(proxy.host.as_ref(), proxy.port),
						clean_host.clone(),
						port,
						Config::default(),
					)
					.await?
				};
			BufStream::new(Box::new(socks_stream) as Box<dyn AsyncReadWrite>)
		}
		None => {
			let tcp_stream = TcpStream::connect(format!("{}:{}", clean_host, port)).await?;
			BufStream::new(Box::new(tcp_stream) as Box<dyn AsyncReadWrite>)
		}
	};

	let mut smtp_transport = SmtpTransport::new(smtp_client, stream).await?;

	// Set "MAIL FROM"
	let from_email = EmailAddress::from_str(&input.from_email).unwrap_or_else(|_| {
		warn!(
			"Invalid 'from_email' provided: '{}'. Using default: 'user@example.org'",
			input.from_email
		);
		EmailAddress::from_str("user@example.org").expect("Default email is valid")
	});
	try_smtp!(
		smtp_transport
			.get_mut()
			.command(MailCommand::new(Some(from_email.into_inner()), vec![]))
			.await,
		smtp_transport,
		input.to_email,
		clean_host,
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

/// Checks deliverability of a target email address using the provided SMTP transport.
async fn check_email_deliverability<S: AsyncBufRead + AsyncWrite + Unpin + Send>(
	smtp_transport: &mut SmtpTransport<S>,
	to_email: &EmailAddress,
) -> Result<Deliverability, SmtpError> {
	match smtp_transport
		.get_mut()
		.command(RcptCommand::new(to_email.clone().into_inner(), vec![]))
		.await
	{
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
		Ok(_) => Ok(Deliverability {
			has_full_inbox: false,
			is_deliverable: true, // response.is_positive()
			is_disabled: false,
		}),
		Err(err) => {
			// We cast to lowercase, because our matched strings below are all
			// lowercase.
			let err_string = err.to_string().to_lowercase();

			// Check if the email account has been disabled or blocked.
			if parser::is_disabled_account(&err_string) {
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: false,
					is_disabled: true,
				});
			}

			// Check if the email account has a full inbox.
			if parser::is_full_inbox(&err_string) {
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
			if parser::is_invalid(&err_string, to_email) {
				return Ok(Deliverability {
					has_full_inbox: false,
					is_deliverable: false,
					is_disabled: false,
				});
			}

			// Return all unparsable errors,.
			Err(SmtpError::AsyncSmtpError(err))
		}
	}
}

/// Checks if the domain has a catch-all email setup.
async fn smtp_is_catch_all<S: AsyncBufRead + AsyncWrite + Unpin + Send>(
	smtp_transport: &mut SmtpTransport<S>,
	domain: &str,
	host: &str,
	input: &CheckEmailInput,
) -> Result<bool, SmtpError> {
	if has_rule(domain, host, &Rule::SkipCatchAll) {
		debug!(
			target: LOG_TARGET,
			"[email={}] Skipping catch-all check for [domain={domain}]",
			input.to_email
		);
		return Ok(false);
	}

	// Create a random 15-char alphanumerical string.
	let mut rng = SmallRng::from_entropy();
	let random_email: String = iter::repeat_with(|| rng.sample(Alphanumeric))
		.map(char::from)
		.take(15)
		.collect();
	let random_email = EmailAddress::new(format!("{}@{}", random_email, domain))?;

	check_email_deliverability(smtp_transport, &random_email)
		.await
		.map(|result| result.is_deliverable)
}

/// Creates an SMTP future for email verification.
async fn create_smtp_future(
	to_email: &EmailAddress,
	host: &str,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
) -> Result<(bool, Deliverability), SmtpError> {
	// FIXME If the SMTP is not connectable, we should actually return an
	// Ok(SmtpDetails { can_connect_smtp: false, ... }).
	let mut smtp_transport = connect_to_smtp_host(host, port, input).await?;

	let is_catch_all = smtp_is_catch_all(&mut smtp_transport, domain, host, input)
		.await
		.unwrap_or(false);
	let deliverability = if is_catch_all {
		Deliverability {
			has_full_inbox: false,
			is_deliverable: true,
			is_disabled: false,
		}
	} else {
		let mut result = check_email_deliverability(&mut smtp_transport, to_email).await;

		// Some SMTP servers automatically close the connection after an error,
		// so we should reconnect to perform a next command.
		//
		// Unfortunately `smtp_transport.is_connected()` doesn't report about this,
		// so we can only check for "io: incomplete" SMTP error being returned.
		// https://github.com/async-email/async-smtp/issues/37
		if let Err(e) = &result {
			if parser::is_err_io_errors(e) {
				debug!(
					target: LOG_TARGET,
					"[email={}] Got `io: incomplete` error, reconnecting.",
					input.to_email
				);

				let _ = smtp_transport.quit().await;
				smtp_transport = connect_to_smtp_host(host, port, input).await?;
				result = check_email_deliverability(&mut smtp_transport, to_email).await;
			}
		}

		result?
	};

	smtp_transport
		.quit()
		.await
		.map_err(SmtpError::AsyncSmtpError)?;

	Ok((is_catch_all, deliverability))
}

/// Get all email details we can from one single `EmailAddress`, without
/// retries.
async fn check_smtp_without_retry(
	to_email: &EmailAddress,
	host: &str,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
) -> Result<SmtpDetails, SmtpError> {
	let fut = create_smtp_future(to_email, host, port, domain, input);

	let (is_catch_all, deliverability) = match input.smtp_timeout {
		Some(smtp_timeout) => {
			let timeout = tokio::time::timeout(smtp_timeout, fut);

			match timeout.await {
				Ok(result) => result?,
				Err(_) => return Err(SmtpError::Timeout(smtp_timeout)),
			}
		}
		None => fut.await?,
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
	host: &str,
	port: u16,
	domain: &str,
	input: &CheckEmailInput,
	count: usize,
) -> Result<SmtpDetails, SmtpError> {
	debug!(
		target: LOG_TARGET,
		email=input.to_email,
		attempt=input.retries - count + 1,
		host=host,
		port=port,
		"Check SMTP"
	);

	let result = check_smtp_without_retry(to_email, host, port, domain, input).await;

	debug!(
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
		Err(SmtpError::HeadlessError(_)) => result,
		Err(SmtpError::YahooError(_)) => result,
		Err(SmtpError::GmailError(_)) => result,
		// Only retry if the SMTP error was unknown.
		Err(err) if err.get_description().is_none() => {
			if count <= 1 {
				result
			} else {
				debug!(
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
