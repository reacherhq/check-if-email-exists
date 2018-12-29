use lettre::smtp::client::net::NetworkStream;
use lettre::smtp::client::InnerClient;
use lettre::smtp::commands::*;
use lettre::smtp::extension::{ClientId, ServerInfo};
use lettre::EmailAddress;
use std::time::Duration;
use trust_dns_resolver::Name;

// Try to send an smtp command, close if fails.
macro_rules! try_smtp (
    ($res: expr, $client: ident) => ({
        match $res {
            Ok(res) => res,
            _ => {
				$client.close();
                return Err(());
            },
        }
    })
);

// Ensure that the error code we get is okay.
macro_rules! ensure_positive (
    ($res: expr, $client: ident) => ({
		if !$res.is_positive() {
			$client.close();
			return Err(());
		}
		$res
    })
);

pub fn email_exists(from: &str, to: &str, host: &Name, port: u16) -> Result<bool, ()> {
	debug!("Connecting to {}:{}...", host, port);
	let mut smtp_client: InnerClient<NetworkStream> = InnerClient::new();

	// Set timeout.
	try_smtp!(
		smtp_client.set_timeout(Some(Duration::new(3, 0))),
		smtp_client
	);

	// Connect to the host.
	try_smtp!(
		smtp_client.connect(&(host.to_utf8().as_str(), port), None),
		smtp_client
	);

	// Send ehlo and get server info.
	let ehlo_response = ensure_positive!(
		try_smtp!(
			smtp_client.command(EhloCommand::new(ClientId::new("localhost".to_string()))),
			smtp_client
		),
		smtp_client
	);
	let server_info = ServerInfo::from_response(&ehlo_response);
	debug!("Server info: {}", server_info.as_ref().unwrap());

	// Send from.
	ensure_positive!(
		try_smtp!(
			smtp_client.command(MailCommand::new(
				Some(EmailAddress::new(from.to_string()).unwrap()),
				vec![],
			)),
			smtp_client
		),
		smtp_client
	);

	// Send to.
	let rctp_response = try_smtp!(
		smtp_client.command(RcptCommand::new(
			EmailAddress::new(to.to_string()).unwrap(),
			vec![],
		)),
		smtp_client
	);

	// Quit.
	smtp_client.close();

	if let Some(message) = rctp_response.first_line() {
		debug!("message {}", message);
		// 250 2.1.5 Recipient e-mail address ok.
		if message.contains("2.1.5") {
			debug!("1.5.2. {}", message);
			return Ok(true);
		}
		// 550 5.1.1 Mailbox does not exist.
		if message.contains("5.1.1") {
			debug!("5.1.1 {}", message);
			return Ok(false);
		}
	}

	Err(())
}
