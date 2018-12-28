extern crate lettre;
extern crate native_tls;
extern crate trust_dns_resolver;

use self::lettre::smtp::client::net::NetworkStream;
use self::lettre::smtp::client::Client;
use self::lettre::smtp::commands::*;
use self::lettre::smtp::extension::{ClientId, Extension, ServerInfo};
use self::lettre::{ClientTlsParameters, EmailAddress};
use self::native_tls::TlsConnector;
use self::trust_dns_resolver::Name;
use std::time::Duration;

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(res) => {
				if !res.is_positive() {
					if let Some(message) = res.first_line() {
						error!("{}", message);
					}
					$client.close();
					return false;
				}
				res
			},
            Err(err) => {
				error!("{}", err);
				$client.close();
                return false;
            },
        }
    })
);

pub fn email_exists(from: &str, to: &str, host: &Name, port: u16) -> bool {
	info!("Connecting to {}:{}...", host, port);
	let mut email_client: Client<NetworkStream> = Client::new();
	if let Err(err) = email_client.set_timeout(Some(Duration::new(1, 0))) {
		error!("{}", err);
		email_client.close();
		return false;
	}

	let tls_builder = TlsConnector::builder();
	let tls_parameters =
		ClientTlsParameters::new(host.to_string(), tls_builder.unwrap().build().unwrap());

	// Connect to the host
	try_smtp!(
		email_client.connect(&(host.to_utf8().as_str(), port), Some(&tls_parameters)),
		email_client
	);

	// Send ehlo and get server info
	let ehlo_response = try_smtp!(
		email_client.command(EhloCommand::new(ClientId::new("localhost".to_string()))),
		email_client
	);
	let server_info = ServerInfo::from_response(&ehlo_response);
	debug!("Server info: {}", server_info.as_ref().unwrap());

	// Send starttls if available
	if server_info
		.as_ref()
		.unwrap()
		.supports_feature(Extension::StartTls)
	{
		try_smtp!(email_client.command(StarttlsCommand), email_client);

		debug!("Connection is encrypted.");
	}

	// Send from
	try_smtp!(
		email_client.command(MailCommand::new(
			Some(EmailAddress::new(from.to_string()).unwrap()),
			vec![],
		)),
		email_client
	);

	// Send to
	try_smtp!(
		email_client.command(RcptCommand::new(
			EmailAddress::new(to.to_string()).unwrap(),
			vec![],
		)),
		email_client
	);

	// Quit
	try_smtp!(email_client.command(QuitCommand), email_client);

	true
}
