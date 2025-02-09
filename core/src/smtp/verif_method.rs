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

//! TODO: This will ultimately be moved to core.

use std::{collections::HashMap, time::Duration};

use crate::util::input_output::CheckEmailInputProxy;
use serde::{Deserialize, Serialize};

use super::{is_gmail, is_hotmail_b2b, is_hotmail_b2c, is_yahoo};

#[derive(Debug, thiserror::Error)]
pub enum VerifMethodError {
	#[error("Invalid proxies: {0}")]
	InvalidProxies(String),
}

/// Reacher categorizes each email into one of the following email providers.
/// This is used to determine the verification method to use for each email
/// provider.
pub enum EmailProvider {
	Gmail,
	HotmailB2B,
	HotmailB2C,
	Yahoo,
	EverythingElse,
}

impl EmailProvider {
	/// Determine the email provider from the MX host.
	pub fn from_mx_host(host: &str) -> Self {
		if is_hotmail_b2c(host) {
			EmailProvider::HotmailB2C
		} else if is_hotmail_b2b(host) {
			EmailProvider::HotmailB2B
		} else if is_gmail(host) {
			EmailProvider::Gmail
		} else if is_yahoo(host) {
			EmailProvider::Yahoo
		} else {
			EmailProvider::EverythingElse
		}
	}
}

type ProxyID = String;

/// The verification method to use for each email provider.
#[derive(Debug, Default, Clone, Deserialize, PartialEq, Serialize)]
pub struct VerifMethod {
	/// Proxies to use for email verification. The key is any unique name for
	/// the proxy, and the value is the proxy itself. For names, we recommend
	/// "proxy1", "proxy2", etc.
	pub proxies: HashMap<ProxyID, CheckEmailInputProxy>,
	/// Verification method for Gmail.
	pub gmail: GmailVerifMethod,
	/// Verification method for Hotmail B2B.
	pub hotmailb2b: HotmailB2BVerifMethod,
	/// Verification method for Hotmail B2C.
	pub hotmailb2c: HotmailB2CVerifMethod,
	/// Verification method for Yahoo.
	pub yahoo: YahooVerifMethod,
	/// Verification method for everything else.
	pub everything_else: EverythingElseVerifMethod,
}

impl VerifMethod {
	pub fn validate_proxies(&self) -> Result<(), VerifMethodError> {
		match &self.gmail {
			GmailVerifMethod::Smtp(c) => match &c.proxy {
				Some(proxy_id) => {
					self.proxies.get(proxy_id).ok_or_else(|| {
						VerifMethodError::InvalidProxies(format!("Invalid Gmail proxy {proxy_id}"))
					})?;
				}
				_ => {}
			},
		};

		match &self.hotmailb2b {
			HotmailB2BVerifMethod::Smtp(c) => match &c.proxy {
				Some(proxy_id) => {
					self.proxies.get(proxy_id).ok_or_else(|| {
						VerifMethodError::InvalidProxies(format!(
							"Invalid Hotmail B2B proxy {proxy_id}"
						))
					})?;
				}
				_ => {}
			},
		};

		match &self.hotmailb2c {
			HotmailB2CVerifMethod::Smtp(c) => match &c.proxy {
				Some(proxy_id) => {
					self.proxies.get(proxy_id).ok_or_else(|| {
						VerifMethodError::InvalidProxies(format!(
							"Invalid Hotmail B2C proxy {proxy_id}"
						))
					})?;
				}
				_ => {}
			},
			_ => {}
		};

		match &self.yahoo {
			YahooVerifMethod::Smtp(c) => match &c.proxy {
				Some(proxy_id) => {
					self.proxies.get(proxy_id).ok_or_else(|| {
						VerifMethodError::InvalidProxies(format!("Invalid Yahoo proxy {proxy_id}"))
					})?;
				}
				_ => {}
			},
			_ => {}
		};

		match &self.everything_else {
			EverythingElseVerifMethod::Smtp(c) => match &c.proxy {
				Some(proxy_id) => {
					self.proxies.get(proxy_id).ok_or_else(|| {
						VerifMethodError::InvalidProxies(format!(
							"Invalid EverythingElse proxy {proxy_id}"
						))
					})?;
				}
				_ => {}
			},
		};

		Ok(())
	}

	/// Get the proxy to use for the email provider. If there is a
	/// configuration error, such as an unconfigured proxy, this will return
	/// an error.
	pub fn get_proxy(&self, email_provider: EmailProvider) -> Option<&CheckEmailInputProxy> {
		match email_provider {
			EmailProvider::Gmail => match &self.gmail {
				GmailVerifMethod::Smtp(c) => c
					.proxy
					.as_ref()
					.and_then(|proxy_id| self.proxies.get(proxy_id)),
			},
			EmailProvider::HotmailB2B => match &self.hotmailb2b {
				HotmailB2BVerifMethod::Smtp(c) => c
					.proxy
					.as_ref()
					.and_then(|proxy_id| self.proxies.get(proxy_id)),
			},
			EmailProvider::HotmailB2C => match &self.hotmailb2c {
				HotmailB2CVerifMethod::Smtp(c) => c
					.proxy
					.as_ref()
					.and_then(|proxy_id| self.proxies.get(proxy_id)),
				_ => None,
			},
			EmailProvider::Yahoo => match &self.yahoo {
				YahooVerifMethod::Smtp(c) => c
					.proxy
					.as_ref()
					.and_then(|proxy_id| self.proxies.get(proxy_id)),
				_ => None,
			},
			EmailProvider::EverythingElse => match &self.everything_else {
				EverythingElseVerifMethod::Smtp(c) => c
					.proxy
					.as_ref()
					.and_then(|proxy_id| self.proxies.get(proxy_id)),
			},
		}
	}
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub enum GmailVerifMethod {
	Smtp(VerifMethodSmtpConfig),
}

impl Default for GmailVerifMethod {
	fn default() -> Self {
		GmailVerifMethod::Smtp(VerifMethodSmtpConfig::default())
	}
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub enum HotmailB2BVerifMethod {
	/// Use Hotmail's SMTP servers to check if an email exists.
	Smtp(VerifMethodSmtpConfig),
}

impl Default for HotmailB2BVerifMethod {
	fn default() -> Self {
		HotmailB2BVerifMethod::Smtp(VerifMethodSmtpConfig::default())
	}
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Serialize)]
pub enum HotmailB2CVerifMethod {
	/// Use Hotmail's password recovery page to check if an email exists.
	///
	/// This assumes you have a WebDriver compatible process running, then pass
	/// its endpoint, usually http://localhost:9515, into the environment
	/// variable RCH_WEBDRIVER_ADDR. We recommend running chromedriver (and not
	/// geckodriver) as it allows parallel requests.
	#[default]
	Headless,
	/// Use Hotmail's SMTP servers to check if an email exists.
	Smtp(VerifMethodSmtpConfig),
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Serialize)]
pub enum YahooVerifMethod {
	/// Use Yahoo's API to check if an email exists.
	Api,
	/// Use Yahoo's password recovery page to check if an email exists.
	///
	/// This assumes you have a WebDriver compatible process running, then pass
	/// its endpoint, usually http://localhost:9515, into the environment
	/// variable RCH_WEBDRIVER_ADDR. We recommend running chromedriver (and not
	/// geckodriver) as it allows parallel requests.
	#[default]
	Headless,
	/// Use Yahoo's SMTP servers to check if an email exists.
	Smtp(VerifMethodSmtpConfig),
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub enum EverythingElseVerifMethod {
	/// Use the SMTP server of the email provider to check if an email exists.
	Smtp(VerifMethodSmtpConfig),
}

impl Default for EverythingElseVerifMethod {
	fn default() -> Self {
		EverythingElseVerifMethod::Smtp(VerifMethodSmtpConfig::default())
	}
}

/// Configuration on the SMTP verification method. If it used mostly as a
/// serializable struct, to be converted into the domain type
/// `VerifMethodSmtp`.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct VerifMethodSmtpConfig {
	/// Email to use in the `MAIL FROM:` SMTP command.
	///
	/// Defaults to "reacher.email@gmail.com", which is an unused addressed
	/// owned by Reacher.
	pub from_email: String,
	/// Name to use in the `EHLO:` SMTP command.
	///
	/// Defaults to "gmail.com" (note: "localhost" is not a FQDN), mostly for
	/// testing purposes. You should set this to a domain you own.
	pub hello_name: String,
	/// Use a proxy to check if an email exists. This proxy must exist in the
	/// `VerifMethod.proxies` field, and it must be a SOCKS5 proxy.
	pub proxy: Option<ProxyID>,
	/// SMTP port to use for email validation. Generally, ports 25, 465, 587
	/// and 2525 are used.
	///
	/// Defaults to 25.
	pub smtp_port: u16,
	/// Add timeout for the SMTP verification step. Set to None if you don't
	/// want to use a timeout. This timeout is per SMTP connection. For
	/// instance, if you set the number of retries to 2, then the total time
	/// for the SMTP verification step can be up to 2 * `smtp_timeout`.
	///
	/// Defaults to None.
	pub smtp_timeout: Option<Duration>,
	/// Number of total SMTP connections to do. Setting to 2 might bypass
	/// greylisting on some servers, but takes more time.
	///
	/// This setting's naming is a bit misleading, as it's not really retries,
	/// but the total number of SMTP connections to do.
	///
	/// Defaults to 1.
	pub retries: usize,
}

impl Default for VerifMethodSmtpConfig {
	fn default() -> Self {
		Self {
			from_email: "reacher@gmail.com".to_string(),
			hello_name: "gmail.com".to_string(),
			proxy: None,
			smtp_port: 25,
			smtp_timeout: None,
			retries: 1,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifMethodSmtp {
	pub config: VerifMethodSmtpConfig,
	pub proxy: Option<CheckEmailInputProxy>,
}

impl VerifMethodSmtp {
	pub fn new(config: VerifMethodSmtpConfig, proxy: Option<CheckEmailInputProxy>) -> Self {
		Self { config, proxy }
	}
}

mod tests {
	use super::*;

	#[test]
	fn test_validate_proxies() {
		let mut proxies = HashMap::new();
		proxies.insert("proxy1".to_string(), CheckEmailInputProxy::default());
		proxies.insert("proxy2".to_string(), CheckEmailInputProxy::default());

		// Test invalid proxies.
		let verif_method = VerifMethod {
			proxies: proxies.clone(),
			gmail: GmailVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy3".to_string()),
				..Default::default()
			}),
			..Default::default()
		};

		assert!(verif_method.validate_proxies().is_err());

		// Test valid proxies.
		let verif_method = VerifMethod {
			proxies,
			gmail: GmailVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy1".to_string()),
				..Default::default()
			}),
			hotmailb2b: HotmailB2BVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy2".to_string()),
				..Default::default()
			}),
			hotmailb2c: HotmailB2CVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy1".to_string()),
				..Default::default()
			}),
			yahoo: YahooVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy2".to_string()),
				..Default::default()
			}),
			..Default::default()
		};

		assert!(verif_method.validate_proxies().is_ok());
	}

	#[test]
	fn test_get_proxy() {
		let mut proxies = HashMap::new();
		proxies.insert("proxy1".to_string(), CheckEmailInputProxy::default());
		proxies.insert("proxy2".to_string(), CheckEmailInputProxy::default());

		let verif_method = VerifMethod {
			proxies: proxies.clone(),
			gmail: GmailVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy1".to_string()),
				..Default::default()
			}),
			hotmailb2b: HotmailB2BVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy2".to_string()),
				..Default::default()
			}),
			hotmailb2c: HotmailB2CVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy1".to_string()),
				..Default::default()
			}),
			yahoo: YahooVerifMethod::Smtp(VerifMethodSmtpConfig {
				proxy: Some("proxy2".to_string()),
				..Default::default()
			}),
			..Default::default()
		};

		assert_eq!(
			verif_method.get_proxy(EmailProvider::Gmail),
			proxies.get("proxy1")
		);
		assert_eq!(
			verif_method.get_proxy(EmailProvider::HotmailB2B),
			proxies.get("proxy2")
		);
		assert_eq!(
			verif_method.get_proxy(EmailProvider::HotmailB2C),
			proxies.get("proxy1")
		);
		assert_eq!(
			verif_method.get_proxy(EmailProvider::Yahoo),
			proxies.get("proxy2")
		);
		assert!(verif_method
			.get_proxy(EmailProvider::EverythingElse)
			.is_none());
	}
}
