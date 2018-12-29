use lettre::smtp::client::net::NetworkStream;
use lettre::smtp::client::InnerClient;
use lettre::smtp::commands::*;
use lettre::smtp::extension::{ClientId, Extension, ServerInfo};
use lettre::{ClientTlsParameters, EmailAddress};
use native_tls::TlsConnector;
use std::time::Duration;
use trust_dns_resolver::Name;

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(res) => {
				if !res.is_positive() {
					if let Some(message) = res.first_line() {
						debug!("Error: {}", message);
					}
					$client.close();
					return false;
				}
				res
			},
            Err(err) => {
				debug!("Error: {}", err);
				$client.close();
                return false;
            },
        }
    })
);

pub fn email_exists(from: &str, to: &str, host: &Name, port: u16) -> bool {
	debug!("Connecting to {}:{}...", host, port);
	let mut email_client: InnerClient<NetworkStream> = InnerClient::new();
	if let Err(err) = email_client.set_timeout(Some(Duration::new(1, 0))) {
		debug!("{}", err);
		email_client.close();
		return false;
	}

	// Connect to the host. Start with insecure connection and use `STARTTLS`
	// when available
	try_smtp!(
		email_client.connect(&(host.to_utf8().as_str(), port), None),
		email_client
	);

	// Send ehlo and get server info
	let ehlo_response = try_smtp!(
		email_client.command(EhloCommand::new(ClientId::new("localhost".to_string()))),
		email_client
	);
	let server_info = ServerInfo::from_response(&ehlo_response);
	debug!("Server info: {}", server_info.as_ref().unwrap());

	// Send `STARTTLS` if available
	if server_info
		.as_ref()
		.unwrap()
		.supports_feature(Extension::StartTls)
	{
		try_smtp!(email_client.command(StarttlsCommand), email_client);

		let tls_builder = TlsConnector::builder();
		let tls_parameters =
			ClientTlsParameters::new(host.to_string(), tls_builder.build().unwrap());
		if let Err(err) = email_client.upgrade_tls_stream(&tls_parameters) {
			debug!("{}", err);
			email_client.close();
			return false;
		}

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
