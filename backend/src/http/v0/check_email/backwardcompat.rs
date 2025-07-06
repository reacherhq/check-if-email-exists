use std::time::Duration;

use check_if_email_exists::smtp::verif_method::{
	HotmailB2CVerifMethod, VerifMethodSmtpConfig, YahooVerifMethod, DEFAULT_PROXY_ID,
};
use serde::{Deserialize, Serialize};

/// These types are for backward compatibility. The previous API allowed for
/// - yahoo_verif_method: Smtp, Headless, Api
/// - hotmailb2c_verif_method: Smtp, Headless
/// - hotmailb2b_verif_method: Smtp
/// - gmail_verif_method: Smtp
///
/// We keep these types for backward compatibility.

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BackwardCompatYahooVerifMethod {
	Api,
	#[default]
	Headless,
	Smtp,
}

impl BackwardCompatYahooVerifMethod {
	pub fn to_yahoo_verif_method(
		&self,
		// If set, this will use the "default" proxy configuration.
		use_default_proxy: bool,
		hello_name: String,
		from_email: String,
		smtp_timeout: Option<Duration>,
		smtp_port: u16,
		retries: usize,
	) -> YahooVerifMethod {
		match self {
			BackwardCompatYahooVerifMethod::Api => YahooVerifMethod::Api,
			BackwardCompatYahooVerifMethod::Headless => YahooVerifMethod::Headless,
			BackwardCompatYahooVerifMethod::Smtp => YahooVerifMethod::Smtp(VerifMethodSmtpConfig {
				from_email,
				hello_name,
				smtp_port,
				smtp_timeout,
				proxy: if use_default_proxy {
					Some(DEFAULT_PROXY_ID.to_string())
				} else {
					None
				},
				retries,
			}),
		}
	}
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BackwardCompatHotmailB2CVerifMethod {
	#[default]
	Headless,
	Smtp,
}

impl BackwardCompatHotmailB2CVerifMethod {
	pub fn to_hotmailb2c_verif_method(
		&self,
		// If set, this will use the "default" proxy configuration.
		use_default_proxy: bool,
		hello_name: String,
		from_email: String,
		smtp_timeout: Option<Duration>,
		smtp_port: u16,
		retries: usize,
	) -> HotmailB2CVerifMethod {
		match self {
			BackwardCompatHotmailB2CVerifMethod::Headless => HotmailB2CVerifMethod::Headless,
			BackwardCompatHotmailB2CVerifMethod::Smtp => {
				HotmailB2CVerifMethod::Smtp(VerifMethodSmtpConfig {
					from_email,
					hello_name,
					smtp_port,
					smtp_timeout,
					proxy: if use_default_proxy {
						Some(DEFAULT_PROXY_ID.to_string())
					} else {
						None
					},
					retries,
				})
			}
		}
	}
}
