use async_recursion::async_recursion;
use async_smtp::commands::{MailCommand, RcptCommand};
use async_smtp::extension::ClientId;
use async_smtp::{SmtpClient, SmtpTransport};
use fast_socks5::client::Config;
use fast_socks5::util::stream::{tcp_connect, tcp_connect_with_timeout};
use fast_socks5::util::target_addr::TargetAddr;
use fast_socks5::{client::Socks5Stream, AuthenticationMethod, Result, Socks5Command};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use std::iter;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream};
use tokio::net::{lookup_host, TcpStream};

use super::parser;
use super::verif_method::VerifMethodSmtp;
use super::{SmtpDetails, SmtpError};
use crate::rules::{has_rule, Rule};
use crate::util::input_output::CheckEmailInputProxy;
use crate::{EmailAddress, LOG_TARGET};

// Define a new trait that combines AsyncRead, AsyncWrite, and Unpin
trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncReadWrite for T {}

/// Try to send an smtp command, close and return Err if fails.
macro_rules! try_smtp (
    ($res: expr, $client: ident, $to_email: expr, $host: expr, $port: expr) => ({
		if let Err(err) = $res {
			tracing::debug!(
				target: LOG_TARGET,
				email=$to_email.to_string(),
				mx_host=$host,
				port=$port,
				error=?err,
				"Closing connection due to error"
			);
			// Try to close the connection, but ignore if there's an error.
			let _ = $client.quit().await;

			return Err(SmtpError::AsyncSmtpError(err));
		}
    })
);

/// Maximum number of resolved proxy addresses to try before giving up.
const MAX_PROXY_ADDR_ATTEMPTS: usize = 3;

/// Open a TCP connection to the first reachable address, trying at most
/// `MAX_PROXY_ADDR_ATTEMPTS` addresses in order.
async fn connect_to_any_addr(
	addrs: &[SocketAddr],
	connect_timeout_s: Option<u64>,
) -> Result<TcpStream, SmtpError> {
	let mut last_err = None;
	for addr in addrs.iter().take(MAX_PROXY_ADDR_ATTEMPTS) {
		let res = match connect_timeout_s {
			None => tcp_connect(addr).await,
			Some(timeout_s) => tcp_connect_with_timeout(addr, timeout_s).await,
		};
		match res {
			Ok(socket) => {
				tracing::info!(
					target: LOG_TARGET,
					proxy_addr=%addr,
					"Connected to proxy address"
				);
				return Ok(socket);
			}
			Err(err) => {
				tracing::debug!(
					target: LOG_TARGET,
					proxy_addr=%addr,
					error=?err,
					"Failed to connect to proxy address, trying next one"
				);
				last_err = Some(err);
			}
		}
	}

	Err(match last_err {
		Some(err) => err.into(),
		None => std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"proxy host resolved to no addresses",
		)
		.into(),
	})
}

/// Connect to the target host through a SOCKS5 proxy.
///
/// The proxy host is resolved here rather than in fast-socks5, which only
/// ever uses the first resolved address. When the proxy hostname resolves to
/// multiple addresses, we shuffle them so connections spread across all
/// endpoints, and fall back to the next address if the TCP connection fails.
async fn connect_to_socks5_proxy(
	proxy: &CheckEmailInputProxy,
	target_host: String,
	target_port: u16,
) -> Result<Socks5Stream<TcpStream>, SmtpError> {
	// Round up, so that timeouts below 1s don't become 0s.
	let connect_timeout_s = proxy.timeout_ms.map(|ms| ms.div_ceil(1000));
	let mut config = Config::default();
	if let Some(timeout_s) = connect_timeout_s {
		config.set_connect_timeout(timeout_s);
	}

	let mut addrs: Vec<SocketAddr> = lookup_host((proxy.host.as_str(), proxy.port))
		.await?
		.collect();
	addrs.shuffle(&mut SmallRng::from_entropy());

	let socket = connect_to_any_addr(&addrs, connect_timeout_s).await?;

	let auth = match (&proxy.username, &proxy.password) {
		(Some(username), Some(password)) => Some(AuthenticationMethod::Password {
			username: username.clone(),
			password: password.clone(),
		}),
		_ => None,
	};
	let mut socks_stream = Socks5Stream::use_stream(socket, auth, config).await?;
	// Pass the domain to the proxy, DNS will be resolved on the proxy side.
	socks_stream
		.request(
			Socks5Command::TCPConnect,
			TargetAddr::Domain(target_host, target_port),
		)
		.await?;

	Ok(socks_stream)
}

/// Connect to an SMTP host and return the configured client transport.
async fn connect_to_smtp_host(
	to_email: &EmailAddress,
	mx_host: &str,
	verif_method: &VerifMethodSmtp,
) -> Result<SmtpTransport<BufStream<Box<dyn AsyncReadWrite>>>, SmtpError> {
	// hostname verification fails if it ends with '.', for example, using
	// SOCKS5 proxies we can `io: incomplete` error.
	let clean_host = mx_host.trim_end_matches('.').to_string();
	let smtp_client = SmtpClient::new()
		.hello_name(ClientId::Domain(verif_method.config.hello_name.to_string()))
		// Sometimes, using socks5 proxy, we get an `io: incomplete` error
		// when using pipelining and sending two consecutive RCPT TO commands.
		.pipelining(false);

	let stream: BufStream<Box<dyn AsyncReadWrite>> = match &verif_method.proxy {
		Some(proxy) => {
			let socks_stream =
				connect_to_socks5_proxy(proxy, clean_host.clone(), verif_method.config.smtp_port)
					.await?;
			BufStream::new(Box::new(socks_stream) as Box<dyn AsyncReadWrite>)
		}
		None => {
			let tcp_stream =
				TcpStream::connect(format!("{}:{}", clean_host, verif_method.config.smtp_port))
					.await?;
			BufStream::new(Box::new(tcp_stream) as Box<dyn AsyncReadWrite>)
		}
	};

	let mut smtp_transport = SmtpTransport::new(smtp_client, stream).await?;

	// Set "MAIL FROM"
	let from_email = EmailAddress::from_str(&verif_method.config.from_email).unwrap_or_else(|_| {
		tracing::warn!(
			target: LOG_TARGET,
			from_email=verif_method.config.from_email,
			"Invalid 'from_email' provided, using default: 'user@example.org'"
		);
		EmailAddress::from_str("user@example.org").expect("Default email is valid")
	});
	try_smtp!(
		smtp_transport
			.get_mut()
			.command(MailCommand::new(Some(from_email.into_inner()), vec![]))
			.await,
		smtp_transport,
		to_email,
		clean_host,
		verif_method.config.smtp_port
	);

	Ok(smtp_transport)
}

/// Description of the deliverability information we can gather from
/// communicating with the SMTP server.
#[derive(Debug)]
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
			// A 421 means the server is closing the session: the reply says
			// nothing about the mailbox, whatever its text. Check the code
			// before any text-based classification.
			if parser::is_service_closing(&err) {
				return Err(SmtpError::AsyncSmtpError(err));
			}

			// We cast to lowercase, because our matched strings below are all
			// lowercase.
			let err_string = err.to_string().to_lowercase();

			// Greylisting is a temporary deferral, it says nothing about the
			// mailbox. Return the error as-is so that the check is retried,
			// and eventually reported as unknown.
			if parser::is_greylisted(&err_string) {
				return Err(SmtpError::AsyncSmtpError(err));
			}

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
	to_email: &EmailAddress,
) -> Result<bool, SmtpError> {
	if has_rule(domain, host, &Rule::SkipCatchAll) {
		tracing::debug!(
			target: LOG_TARGET,
			email=to_email.to_string(),
			domain=domain,
			"Skipping catch-all check"
		);
		return Ok(false);
	}

	// Create a random 15-char alphanumerical string.
	let mut rng = SmallRng::from_entropy();
	let random_email: String = iter::repeat_with(|| rng.sample(Alphanumeric))
		.map(char::from)
		.take(15)
		.collect();
	let random_email = EmailAddress::new(format!("{random_email}@{domain}"))?;

	check_email_deliverability(smtp_transport, &random_email)
		.await
		.map(|result| result.is_deliverable)
}

/// Creates an SMTP future for email verification.
async fn create_smtp_future(
	to_email: &EmailAddress,
	mx_host: &str,
	domain: &str,
	verif_method: &VerifMethodSmtp,
) -> Result<(bool, Deliverability), SmtpError> {
	// FIXME If the SMTP is not connectable, we should actually return an
	// Ok(SmtpDetails { can_connect_smtp: false, ... }).
	let mut smtp_transport = connect_to_smtp_host(to_email, mx_host, verif_method).await?;

	let is_catch_all = match smtp_is_catch_all(&mut smtp_transport, domain, mx_host, to_email).await
	{
		Ok(is_catch_all) => is_catch_all,
		// The server closed the session on the catch-all probe. Sending the
		// real RCPT TO on it would only fail with `io: incomplete` and hide
		// this reply, so return it now. Any retry happens on a new session.
		//
		// Don't send QUIT: the session is already closed, and waiting for a
		// reply that may never come would replace this error with a timeout,
		// or hang if no timeout is set. Dropping the transport closes it.
		Err(SmtpError::AsyncSmtpError(err)) if parser::is_service_closing(&err) => {
			return Err(SmtpError::AsyncSmtpError(err));
		}
		Err(err) => {
			tracing::debug!(
				target: LOG_TARGET,
				email=to_email.to_string(),
				error=?err,
				"Catch-all check failed, assuming not catch-all"
			);
			false
		}
	};
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
		//
		// Other servers limit the number of recipients per session, and
		// answer "452 4.5.3 Too many recipients" here because the catch-all
		// probe already used the session's quota. The real RCPT TO must then
		// be sent on a new session too.
		if let Err(e) = &result {
			if parser::is_err_io_errors(e) || parser::is_err_too_many_recipients(e) {
				tracing::debug!(
					target: LOG_TARGET,
					email=to_email.to_string(),
					error=?e,
					"Got `io: incomplete` or too many recipients error, reconnecting"
				);

				let _ = smtp_transport.quit().await;
				result = match connect_to_smtp_host(to_email, mx_host, verif_method).await {
					Ok(new_transport) => {
						smtp_transport = new_transport;
						check_email_deliverability(&mut smtp_transport, to_email).await
					}
					// The server replied while setting up the new session (e.g.
					// "550 IP blacklisted" to MAIL FROM): that reply is the
					// actionable error.
					Err(reconnect_err) if has_smtp_reply(&reconnect_err) => Err(reconnect_err),
					// Otherwise keep the original error: a reconnect that got
					// no reply is secondary and would otherwise mask it.
					Err(reconnect_err) => {
						tracing::debug!(
							target: LOG_TARGET,
							email=to_email.to_string(),
							error=?reconnect_err,
							"Reconnect failed, keeping original error"
						);
						result
					}
				};
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
	mx_host: &str,
	domain: &str,
	verif_method: &VerifMethodSmtp,
) -> Result<SmtpDetails, SmtpError> {
	let fut = create_smtp_future(to_email, mx_host, domain, verif_method);

	let (is_catch_all, deliverability) = match verif_method.config.smtp_timeout {
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
	mx_host: &str,
	domain: &str,
	verif_method: &VerifMethodSmtp,
	// Number of remaining retries.
	count: usize,
) -> Result<SmtpDetails, SmtpError> {
	tracing::debug!(
		target: LOG_TARGET,
		email=to_email.to_string(),
		attempt=verif_method.config.retries - count + 1,
		mx_host=mx_host,
		port=verif_method.config.smtp_port,
		using_proxy=verif_method.proxy.is_some(),
		"Check SMTP"
	);

	let result = check_smtp_without_retry(to_email, mx_host, domain, verif_method).await;

	tracing::debug!(
		target: LOG_TARGET,
		email=to_email.to_string(),
		attempt=verif_method.config.retries - count + 1,
		mx_host=mx_host,
		port=verif_method.config.smtp_port,
		result=?result,
		"Got SMTP check result"
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
				tracing::debug!(
					target: LOG_TARGET,
					email=to_email.to_string(),
					"Potential greylisting detected, retrying"
				);
				let retry_result =
					check_smtp_with_retry(to_email, mx_host, domain, verif_method, count - 1).await;
				match retry_result {
					// The retry got no reply from the SMTP server (proxy or
					// connection failure, EOF before the greeting, timeout...),
					// so the server's reply from this attempt is the more
					// useful error to report.
					Err(retry_err)
						if matches!(&result, Err(err) if has_smtp_reply(err))
							&& !has_smtp_reply(&retry_err) =>
					{
						tracing::debug!(
							target: LOG_TARGET,
							email=to_email.to_string(),
							error=?retry_err,
							"Retry got no SMTP reply, keeping original SMTP error"
						);
						result
					}
					_ => retry_result,
				}
			}
		}
		_ => result,
	}
}

/// Whether the error carries an actual 4xx/5xx reply from the SMTP server, as
/// opposed to a failure to talk to it (connection, proxy, I/O, timeout...).
fn has_smtp_reply(err: &SmtpError) -> bool {
	matches!(
		err,
		SmtpError::AsyncSmtpError(
			async_smtp::error::Error::Transient(_) | async_smtp::error::Error::Permanent(_)
		)
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use tokio::io::{AsyncReadExt, AsyncWriteExt};
	use tokio::net::TcpListener;

	/// Returns an address on which nothing is listening.
	async fn closed_addr() -> SocketAddr {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		listener.local_addr().unwrap()
	}

	#[tokio::test]
	async fn connect_to_any_addr_falls_back_to_next_address() {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let open = listener.local_addr().unwrap();
		let closed = closed_addr().await;

		let socket = connect_to_any_addr(&[closed, open], Some(1)).await.unwrap();
		assert_eq!(socket.peer_addr().unwrap(), open);
	}

	#[tokio::test]
	async fn connect_to_any_addr_fails_when_all_unreachable() {
		let addrs = [closed_addr().await, closed_addr().await];
		assert!(matches!(
			connect_to_any_addr(&addrs, Some(1)).await,
			Err(SmtpError::Socks5(_))
		));
	}

	#[tokio::test]
	async fn connect_to_any_addr_fails_on_empty_addrs() {
		assert!(matches!(
			connect_to_any_addr(&[], None).await,
			Err(SmtpError::IOError(_))
		));
	}

	/// Scripted behavior of one session of the mock SMTP server.
	enum MockSession {
		/// Accept the TCP connection, then close it before the 220 greeting.
		CloseBeforeGreeting,
		/// MAIL FROM gets `mail_reply`. Each RCPT TO gets the next reply from
		/// `rcpt_replies`; once they're exhausted, the connection is closed
		/// without a reply. After a 421, the connection is closed, unless
		/// `hang_after_421` is set: then it stays open, unanswered.
		Smtp {
			mail_reply: &'static str,
			rcpt_replies: Vec<&'static str>,
			hang_after_421: bool,
		},
	}

	fn smtp_session(rcpt_replies: Vec<&'static str>) -> MockSession {
		MockSession::Smtp {
			mail_reply: "250 OK",
			rcpt_replies,
			hang_after_421: false,
		}
	}

	/// Spawn a mock SMTP server that plays `sessions` on successive
	/// connections, then refuses further connections. Returns the port and,
	/// for each session, the commands received.
	async fn mock_smtp_server(
		sessions: Vec<MockSession>,
	) -> (u16, tokio::task::JoinHandle<Vec<Vec<String>>>) {
		use tokio::io::AsyncBufReadExt;

		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let port = listener.local_addr().unwrap().port();
		let handle = tokio::spawn(async move {
			let mut all_received = vec![];
			for session in sessions {
				let (socket, _) = listener.accept().await.unwrap();
				let (mail_reply, rcpt_replies, hang_after_421) = match session {
					MockSession::CloseBeforeGreeting => {
						drop(socket);
						all_received.push(vec![]);
						continue;
					}
					MockSession::Smtp {
						mail_reply,
						rcpt_replies,
						hang_after_421,
					} => (mail_reply, rcpt_replies, hang_after_421),
				};
				let mut socket = tokio::io::BufStream::new(socket);
				let mut replies = rcpt_replies.into_iter();
				let mut received = vec![];
				let mut hanging = false;

				socket.write_all(b"220 mock ESMTP\r\n").await.unwrap();
				socket.flush().await.unwrap();
				loop {
					let mut line = String::new();
					match socket.read_line(&mut line).await {
						Ok(0) | Err(_) => break,
						Ok(_) => {}
					}
					let line = line.trim_end().to_string();
					let cmd = line.to_uppercase();
					received.push(line);
					if hanging {
						continue;
					}
					let reply = if cmd.starts_with("EHLO") {
						"250 OK"
					} else if cmd.starts_with("MAIL") {
						mail_reply
					} else if cmd.starts_with("RCPT") {
						match replies.next() {
							Some(reply) => reply,
							None => break,
						}
					} else if cmd.starts_with("QUIT") {
						let _ = socket.write_all(b"221 Bye\r\n").await;
						let _ = socket.flush().await;
						break;
					} else {
						"500 Unknown command"
					};
					socket
						.write_all(format!("{reply}\r\n").as_bytes())
						.await
						.unwrap();
					socket.flush().await.unwrap();
					if reply.starts_with("421") {
						if hang_after_421 {
							hanging = true;
						} else {
							break;
						}
					}
				}
				all_received.push(received);
			}
			all_received
		});
		(port, handle)
	}

	fn mock_verif_method(port: u16, retries: usize) -> VerifMethodSmtp {
		VerifMethodSmtp::new(
			super::super::verif_method::VerifMethodSmtpConfig {
				smtp_port: port,
				retries,
				..Default::default()
			},
			None,
		)
	}

	fn count_rcpt(received: &[String]) -> usize {
		received
			.iter()
			.filter(|l| l.to_uppercase().starts_with("RCPT"))
			.count()
	}

	fn assert_is_421(res: Result<impl std::fmt::Debug, SmtpError>, context: &str) {
		match res {
			Err(SmtpError::AsyncSmtpError(err)) => {
				assert!(parser::is_service_closing(&err), "{}: {:?}", context, err)
			}
			other => panic!("{}: expected 421 error, got {:?}", context, other),
		}
	}

	// The second reply text would match `parser::is_invalid` if it were
	// classified by text, so this also checks we rely on the 421 code.
	const REPLIES_421: [&str; 2] = [
		"421 4.4.1 Service temporarily unavailable",
		"421 4.7.0 Recipient address rejected: try again later",
	];

	#[tokio::test]
	async fn catch_all_421_is_returned_without_sending_real_rcpt() {
		for reply in REPLIES_421 {
			let (port, server) = mock_smtp_server(vec![smtp_session(vec![reply])]).await;
			let to_email = EmailAddress::from_str("real@example.org").unwrap();

			let res = create_smtp_future(
				&to_email,
				"127.0.0.1",
				"example.org",
				&mock_verif_method(port, 1),
			)
			.await;

			assert_is_421(res, reply);
			let received = server.await.unwrap().remove(0);
			assert_eq!(count_rcpt(&received), 1, "{reply}: {received:?}");
			assert!(!received.iter().any(|l| l.contains("real@example.org")));
		}
	}

	#[tokio::test]
	async fn catch_all_421_is_returned_when_server_keeps_socket_open() {
		// The server answers 421 but never closes the socket nor answers
		// further commands. No SMTP timeout is set, so waiting for any reply
		// would hang.
		let (port, server) = mock_smtp_server(vec![MockSession::Smtp {
			mail_reply: "250 OK",
			rcpt_replies: vec![REPLIES_421[0]],
			hang_after_421: true,
		}])
		.await;
		let to_email = EmailAddress::from_str("real@example.org").unwrap();

		let res = tokio::time::timeout(
			std::time::Duration::from_secs(5),
			create_smtp_future(
				&to_email,
				"127.0.0.1",
				"example.org",
				&mock_verif_method(port, 1),
			),
		)
		.await
		.expect("should return the 421 without waiting on the server");

		assert_is_421(res, REPLIES_421[0]);
		let received = server.await.unwrap().remove(0);
		assert_eq!(count_rcpt(&received), 1, "{received:?}");
	}

	#[tokio::test]
	async fn catch_all_421_survives_failed_retry() {
		for reply in REPLIES_421 {
			// Retry is refused at TCP level, or accepted and closed before the
			// SMTP greeting.
			for close_before_greeting in [false, true] {
				let mut sessions = vec![smtp_session(vec![reply])];
				if close_before_greeting {
					sessions.push(MockSession::CloseBeforeGreeting);
				}
				let (port, server) = mock_smtp_server(sessions).await;
				let to_email = EmailAddress::from_str("real@example.org").unwrap();

				let res = check_smtp_with_retry(
					&to_email,
					"127.0.0.1",
					"example.org",
					&mock_verif_method(port, 2),
					2,
				)
				.await;

				let context = format!("{reply} (close_before_greeting={close_before_greeting})");
				assert_is_421(res, &context);
				server.await.unwrap();
			}
		}
	}

	#[tokio::test]
	async fn failed_reconnect_keeps_original_error() {
		// Catch-all probe is rejected, then the server drops the connection on
		// the real RCPT TO, and refuses the reconnect.
		let (port, server) =
			mock_smtp_server(vec![smtp_session(vec!["550 5.1.1 User unknown"])]).await;
		let to_email = EmailAddress::from_str("real@example.org").unwrap();

		let res = create_smtp_future(
			&to_email,
			"127.0.0.1",
			"example.org",
			&mock_verif_method(port, 1),
		)
		.await;

		match res {
			Err(err) => assert!(parser::is_err_io_errors(&err), "{:?}", err),
			other => panic!("expected `io: incomplete` error, got {:?}", other),
		}
		server.await.unwrap();
	}

	#[tokio::test]
	async fn reconnect_returns_server_reply() {
		// Same as above, but the reconnect reaches the server, which rejects
		// MAIL FROM because our IP is blacklisted: that reply must win.
		let (port, server) = mock_smtp_server(vec![
			smtp_session(vec!["550 5.1.1 User unknown"]),
			MockSession::Smtp {
				mail_reply: "550 5.7.1 Client host [127.0.0.1] blacklisted",
				rcpt_replies: vec![],
				hang_after_421: false,
			},
		])
		.await;
		let to_email = EmailAddress::from_str("real@example.org").unwrap();

		let res = create_smtp_future(
			&to_email,
			"127.0.0.1",
			"example.org",
			&mock_verif_method(port, 1),
		)
		.await;

		match res {
			Err(err) => assert!(
				matches!(
					err.get_description(),
					Some(super::super::SmtpErrorDesc::IpBlacklisted)
				),
				"{:?}",
				err
			),
			other => panic!("expected IP blacklisted error, got {:?}", other),
		}
		server.await.unwrap();
	}

	#[tokio::test]
	async fn too_many_recipients_reconnects_for_real_rcpt() {
		// The server accepts a single RCPT TO per session: the catch-all probe
		// uses it, so the real RCPT TO must be sent on a new session.
		const TOO_MANY: &str = "452 4.5.3 Too many recipients";
		for (real_reply, is_deliverable) in [("250 OK", true), ("550 5.1.1 User unknown", false)] {
			let (port, server) = mock_smtp_server(vec![
				smtp_session(vec!["550 5.1.1 User unknown", TOO_MANY]),
				smtp_session(vec![real_reply]),
			])
			.await;
			let to_email = EmailAddress::from_str("real@example.org").unwrap();

			let (is_catch_all, deliverability) = create_smtp_future(
				&to_email,
				"127.0.0.1",
				"example.org",
				&mock_verif_method(port, 1),
			)
			.await
			.unwrap_or_else(|err| panic!("{}: {:?}", real_reply, err));

			assert!(!is_catch_all);
			assert_eq!(deliverability.is_deliverable, is_deliverable);
			// The new session only gets the real RCPT TO, no catch-all probe.
			let received = server.await.unwrap();
			assert_eq!(count_rcpt(&received[1]), 1, "{received:?}");
			assert!(received[1].iter().any(|l| l.contains("real@example.org")));
		}
	}

	#[tokio::test]
	async fn too_many_recipients_is_returned_when_reconnect_fails() {
		// The reconnect is refused: the 452 is still the error to report.
		let (port, server) = mock_smtp_server(vec![smtp_session(vec![
			"550 5.1.1 User unknown",
			"452 4.5.3 Too many recipients",
		])])
		.await;
		let to_email = EmailAddress::from_str("real@example.org").unwrap();

		let res = create_smtp_future(
			&to_email,
			"127.0.0.1",
			"example.org",
			&mock_verif_method(port, 1),
		)
		.await;

		match res {
			Err(err) => assert!(parser::is_err_too_many_recipients(&err), "{:?}", err),
			other => panic!("expected too many recipients error, got {:?}", other),
		}
		server.await.unwrap();
	}

	#[tokio::test]
	async fn connect_to_socks5_proxy_requests_target_domain() {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let proxy_port = listener.local_addr().unwrap().port();

		// Minimal SOCKS5 server: accept "no auth", then read the CONNECT
		// request and return the requested domain.
		let server = tokio::spawn(async move {
			let (mut socket, _) = listener.accept().await.unwrap();
			let mut greeting = [0u8; 3];
			socket.read_exact(&mut greeting).await.unwrap();
			assert_eq!(greeting, [5, 1, 0]);
			socket.write_all(&[5, 0]).await.unwrap();

			let mut header = [0u8; 5];
			socket.read_exact(&mut header).await.unwrap();
			assert_eq!(header[..4], [5, 1, 0, 3]);
			let mut domain = vec![0u8; header[4] as usize];
			socket.read_exact(&mut domain).await.unwrap();
			let mut port = [0u8; 2];
			socket.read_exact(&mut port).await.unwrap();
			socket
				.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])
				.await
				.unwrap();
			(String::from_utf8(domain).unwrap(), u16::from_be_bytes(port))
		});

		let proxy = CheckEmailInputProxy {
			host: "127.0.0.1".into(),
			port: proxy_port,
			username: None,
			password: None,
			timeout_ms: Some(500),
		};
		connect_to_socks5_proxy(&proxy, "mx.example.org".into(), 25)
			.await
			.unwrap();
		assert_eq!(server.await.unwrap(), ("mx.example.org".into(), 25));
	}
}
