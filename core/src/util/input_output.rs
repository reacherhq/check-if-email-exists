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

use std::env;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use async_smtp::{ClientSecurity, ClientTlsParameters};
use chrono::{DateTime, Utc};
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};

use crate::misc::{MiscDetails, MiscError};
use crate::mx::{MxDetails, MxError};
use crate::smtp::{SmtpDebug, SmtpDetails, SmtpError, SmtpErrorDesc};
use crate::syntax::SyntaxDetails;

/// Perform the email verification via a specified proxy. The usage of a proxy
/// is optional.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct CheckEmailInputProxy {
	/// Use the specified SOCKS5 proxy host to perform email verification.
	pub host: String,
	/// Use the specified SOCKS5 proxy port to perform email verification.
	pub port: u16,
	/// Username to pass to proxy authentication.
	pub username: Option<String>,
	/// Password to pass to proxy authentication.
	pub password: Option<String>,
}

/// Define how to apply TLS to a SMTP client connection. Will be converted into
/// async_smtp::ClientSecurity.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SmtpSecurity {
	/// Insecure connection only (for testing purposes).
	None,
	/// Start with insecure connection and use `STARTTLS` when available.
	Opportunistic,
	/// Start with insecure connection and require `STARTTLS`.
	Required,
	/// Use TLS wrapped connection.
	Wrapper,
}

impl Default for SmtpSecurity {
	fn default() -> Self {
		Self::Opportunistic
	}
}

impl SmtpSecurity {
	pub fn to_client_security(self, tls_params: ClientTlsParameters) -> ClientSecurity {
		match self {
			Self::None => ClientSecurity::None,
			Self::Opportunistic => ClientSecurity::Opportunistic(tls_params),
			Self::Required => ClientSecurity::Required(tls_params),
			Self::Wrapper => ClientSecurity::Wrapper(tls_params),
		}
	}
}

/// Select how to verify Yahoo emails.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum YahooVerifMethod {
	/// Use Yahoo's API to check if an email exists.
	Api,
	/// Use Yahoo's password recovery page to check if an email exists.
	///
	/// This assumes you have a WebDriver compatible process running, then pass
	/// its endpoint, usually http://localhost:9515, into the environment
	/// variable RCH_WEBDRIVER_ADDR. We recommend running chromedriver (and not
	/// geckodriver) as it allows parallel requests.
	#[cfg(feature = "headless")]
	Headless,
	/// Use Yahoo's SMTP servers to check if an email exists.
	Smtp,
}

impl FromStr for YahooVerifMethod {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Api" => Ok(Self::Api),
			#[cfg(feature = "headless")]
			"Headless" => Ok(Self::Headless),
			"Smtp" => Ok(Self::Smtp),
			_ => Err(format!("Unknown yahoo verify method: {}", s)),
		}
	}
}

/// Select how to verify Gmail emails.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum GmailVerifMethod {
	/// Use Gmail's API to check if an email exists.
	Api,
	/// Use Gmail's SMTP servers to check if an email exists.
	Smtp,
}

impl FromStr for GmailVerifMethod {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Api" => Ok(Self::Api),
			"Smtp" => Ok(Self::Smtp),
			_ => Err(format!("Unknown gmail verify method: {}", s)),
		}
	}
}

/// Select how to verify Hotmail emails.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum HotmailVerifMethod {
	/// Use OneDrive API to check if an email exists.
	OneDriveApi,
	/// Use Hotmail's password recovery page to check if an email exists.
	///
	/// This assumes you have a WebDriver compatible process running, then pass
	/// its endpoint, usually http://localhost:9515, into the environment
	/// variable RCH_WEBDRIVER_ADDR. We recommend running chromedriver (and not
	/// geckodriver) as it allows parallel requests.
	#[cfg(feature = "headless")]
	Headless,
	/// Use Hotmail's SMTP servers to check if an email exists.
	Smtp,
}

impl FromStr for HotmailVerifMethod {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"OneDriveApi" => Ok(Self::OneDriveApi),
			#[cfg(feature = "headless")]
			"Headless" => Ok(Self::Headless),
			"Smtp" => Ok(Self::Smtp),
			_ => Err(format!("Unknown hotmail verify method: {}", s)),
		}
	}
}

/// Builder pattern for the input argument into the main `email_exists`
/// function.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CheckEmailInput {
	/// The email to validate.
	pub to_email: String,
	/// Email to use in the `MAIL FROM:` SMTP command.
	///
	/// Defaults to "reacher.email@gmail.com", which is an unused addressed
	/// owned by Reacher.
	pub from_email: String,
	/// Name to use in the `EHLO:` SMTP command.
	///
	/// Defaults to "gmail.com" (note: "localhost" is not a FQDN).
	pub hello_name: String,
	/// Perform the email verification via the specified SOCK5 proxy. The usage of a
	/// proxy is optional.
	pub proxy: Option<CheckEmailInputProxy>,
	/// SMTP port to use for email validation. Generally, ports 25, 465, 587
	/// and 2525 are used.
	///
	/// Defaults to 25.
	pub smtp_port: u16,
	/// Add timeout for the SMTP verification step. Set to None if you don't
	/// want to use a timeout.
	///
	/// Defaults to 30s.
	pub smtp_timeout: Option<Duration>,
	/// Select how to verify Yahoo emails.
	///
	/// Defaults to Headless.
	pub yahoo_verif_method: YahooVerifMethod,
	/// Select how to verify Gmail addresses.
	///
	/// Defaults to Smtp.
	pub gmail_verif_method: GmailVerifMethod,
	/// Select how to verify Hotmail/Outlook/Microsoft email addresses.
	///
	/// Defaults to Headless.
	pub hotmail_verif_method: HotmailVerifMethod,
	// Whether to check if a gravatar image is existing for the given email.
	//
	// Defaults to false.
	pub check_gravatar: bool,
	/// Check if a the email address is present in HaveIBeenPwned API.
	// If the api_key is filled, HaveIBeenPwned API is checked
	pub haveibeenpwned_api_key: Option<String>,
	/// Number of retries of SMTP connections to do. Setting to 2 might bypass
	/// greylisting on some servers, but takes more time.
	///
	/// Defaults to 1.
	pub retries: usize,
	/// How to apply TLS to a SMTP client connection.
	///
	/// Defaults to Opportunistic.
	pub smtp_security: SmtpSecurity,
	/// **IMPORTANT:** This is a beta feature, and might be completely removed,
	/// or moved somewhere else, before the next release.
	///
	/// List of domains to skip when doing an SMTP connection, because we know
	/// they return "unknown". For each string in this list, we check if the MX
	/// record **contains** the string; if yes, we return an error saying the
	/// SMTP verification is skipped.
	///
	/// Related issue: https://github.com/reacherhq/check-if-email-exists/issues/937
	///
	/// ## Example
	///
	/// If you want to skip Zoho emails, it's good enough to ".zoho.com." to
	/// the list. This way it will skip all Zoho emails as their MX records
	/// show "mx{N}.zoho.com.". Simply putting "zoho" might give false
	/// positives if you have an email provider like "mycustomzoho.com".
	///
	/// Defaults to: [""]
	pub skipped_domains: Vec<String>,
}

impl Default for CheckEmailInput {
	fn default() -> Self {
		CheckEmailInput {
			to_email: "".into(),
			from_email: env::var("RCH_FROM_EMAIL")
				.unwrap_or_else(|_| "reacher.email@gmail.com".into()), // Unused, owned by Reacher
			hello_name: env::var("RCH_HELLO_NAME").unwrap_or_else(|_| "gmail.com".into()),
			proxy: None,
			smtp_port: 25,
			smtp_security: SmtpSecurity::default(),
			smtp_timeout: env::var("RCH_SMTP_TIMEOUT")
				.ok()
				.and_then(|s| s.parse::<u64>().ok())
				.map(Duration::from_secs)
				.or(Some(Duration::from_secs(30))),
			#[cfg(not(feature = "headless"))]
			yahoo_verif_method: YahooVerifMethod::Api,
			#[cfg(feature = "headless")]
			yahoo_verif_method: YahooVerifMethod::Headless,
			gmail_verif_method: GmailVerifMethod::Smtp,
			#[cfg(not(feature = "headless"))]
			hotmail_verif_method: HotmailVerifMethod::OneDriveApi,
			#[cfg(feature = "headless")]
			hotmail_verif_method: HotmailVerifMethod::Headless,
			check_gravatar: false,
			haveibeenpwned_api_key: None,
			retries: 1,
			skipped_domains: vec![],
		}
	}
}

impl CheckEmailInput {
	/// Create a new CheckEmailInput.
	pub fn new(to_email: String) -> CheckEmailInput {
		CheckEmailInput {
			to_email,
			..Default::default()
		}
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command. Defaults to
	/// `user@example.org` if not explicitly set.
	#[deprecated(since = "0.8.24", note = "Please use set_from_email instead")]
	pub fn from_email(&mut self, email: String) -> &mut CheckEmailInput {
		self.from_email = email;
		self
	}

	/// Set the email to use in the `MAIL FROM:` SMTP command. Defaults to
	/// `user@example.org` if not explicitly set.
	pub fn set_from_email(&mut self, email: String) -> &mut CheckEmailInput {
		self.from_email = email;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command. Defaults to `localhost`
	/// if not explicitly set.
	#[deprecated(since = "0.8.24", note = "Please use set_hello_name instead")]
	pub fn hello_name(&mut self, name: String) -> &mut CheckEmailInput {
		self.hello_name = name;
		self
	}

	/// Set the name to use in the `EHLO:` SMTP command. Defaults to `localhost`
	/// if not explicitly set.
	pub fn set_hello_name(&mut self, name: String) -> &mut CheckEmailInput {
		self.hello_name = name;
		self
	}

	/// Use the specified SOCK5 proxy to perform email verification.
	#[deprecated(since = "0.8.24", note = "Please use set_proxy instead")]
	pub fn proxy(&mut self, proxy_host: String, proxy_port: u16) -> &mut CheckEmailInput {
		self.proxy = Some(CheckEmailInputProxy {
			host: proxy_host,
			port: proxy_port,
			..Default::default()
		});
		self
	}

	/// Use the specified SOCK5 proxy to perform email verification.
	pub fn set_proxy(&mut self, proxy: CheckEmailInputProxy) -> &mut CheckEmailInput {
		self.proxy = Some(proxy);
		self
	}

	/// Set the number of SMTP retries to do.
	pub fn set_retries(&mut self, retries: usize) -> &mut CheckEmailInput {
		self.retries = retries;
		self
	}

	/// Add optional timeout for the SMTP verification step.
	#[deprecated(since = "0.8.24", note = "Please use set_smtp_timeout instead")]
	pub fn smtp_timeout(&mut self, duration: Duration) -> &mut CheckEmailInput {
		self.smtp_timeout = Some(duration);
		self
	}

	/// Change the SMTP port.
	pub fn set_smtp_port(&mut self, port: u16) -> &mut CheckEmailInput {
		self.smtp_port = port;
		self
	}

	/// Set the SMTP client security to use for TLS.
	pub fn set_smtp_security(&mut self, smtp_security: SmtpSecurity) -> &mut CheckEmailInput {
		self.smtp_security = smtp_security;
		self
	}

	/// Add optional timeout for the SMTP verification step. This is the
	/// timeout for _each_ SMTP connection attempt, not for the whole email
	/// verification process.
	pub fn set_smtp_timeout(&mut self, duration: Option<Duration>) -> &mut CheckEmailInput {
		self.smtp_timeout = duration;
		self
	}

	/// Set whether to use Yahoo's API, headless navigator, or connecting
	/// directly to their SMTP servers. Defaults to Headless.
	pub fn set_yahoo_verif_method(
		&mut self,
		verif_method: YahooVerifMethod,
	) -> &mut CheckEmailInput {
		self.yahoo_verif_method = verif_method;
		self
	}

	/// Set whether to use Gmail's API or connecting directly to their SMTP
	/// servers. Defaults to false.
	pub fn set_gmail_verif_method(
		&mut self,
		verif_method: GmailVerifMethod,
	) -> &mut CheckEmailInput {
		self.gmail_verif_method = verif_method;
		self
	}

	/// Set whether to use Microsoft 365's OneDrive API, a headless navigator,
	/// or connecting directly to their SMTP servers for hotmail addresse.
	/// Defaults to Headless.
	pub fn set_hotmail_verif_method(
		&mut self,
		verif_method: HotmailVerifMethod,
	) -> &mut CheckEmailInput {
		self.hotmail_verif_method = verif_method;
		self
	}

	/// Whether to check if a gravatar image is existing for the given email.
	/// Defaults to false.
	pub fn set_check_gravatar(&mut self, check_gravatar: bool) -> &mut CheckEmailInput {
		self.check_gravatar = check_gravatar;
		self
	}

	/// Whether to haveibeenpwned' API for the given email
	/// check only if the api_key is set.
	pub fn set_haveibeenpwned_api_key(&mut self, api_key: Option<String>) -> &mut CheckEmailInput {
		self.haveibeenpwned_api_key = api_key;
		self
	}

	/// **IMPORTANT:** This is a beta feature, and might be completely removed,
	/// or moved somewhere else, before the next release.
	///
	/// List of domains to skip when doing an SMTP connection, because we know
	/// they return "unknown".
	pub fn set_skipped_domains(&mut self, domains: Vec<String>) -> &mut CheckEmailInput {
		self.skipped_domains = domains;
		self
	}
}

/// An enum to describe how confident we are that the recipient address is
/// real.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reachable {
	/// The email is safe to send.
	Safe,
	/// The email address appears to exist, but has quality issues that may
	/// result in low engagement or a bounce. Emails are classified as risky
	/// when one of the following happens:
	/// - catch-all email,
	/// - disposable email,
	/// - role-based address,
	/// - full inbox.
	Risky,
	/// Emails that don't exist or are syntactically incorrect. Do not send to
	/// these emails.
	Invalid,
	/// We're unable to get a valid response from the recipient's email server.
	Unknown,
}

/// Details about the email verification used for debugging.
#[derive(Debug, Deserialize, Serialize)]
pub struct DebugDetails {
	/// The name of the server that performed the email verification.
	/// It's generally passed as an environment variable RCH_BACKEND_NAME.
	pub server_name: String,
	/// The time when the email verification started.
	pub start_time: DateTime<Utc>,
	/// The time when the email verification ended.
	pub end_time: DateTime<Utc>,
	/// The duration of the email verification.
	pub duration: Duration,
	/// Details about the email verification used for debugging.
	pub smtp: SmtpDebug,
}

impl Default for DebugDetails {
	fn default() -> Self {
		Self {
			server_name: env::var("RCH_BACKEND_NAME").unwrap_or_else(|_| String::new()),
			start_time: SystemTime::now().into(),
			end_time: SystemTime::now().into(),
			duration: Duration::default(),
			smtp: SmtpDebug::default(),
		}
	}
}

/// The result of the [check_email](check_email) function.
#[derive(Debug)]
pub struct CheckEmailOutput {
	/// Input by the user.
	pub input: String,
	pub is_reachable: Reachable,
	/// Misc details about the email address.
	pub misc: Result<MiscDetails, MiscError>,
	/// Details about the MX host.
	pub mx: Result<MxDetails, MxError>,
	/// Details about the SMTP responses of the email.
	pub smtp: Result<SmtpDetails, SmtpError>,
	/// Details about the email address.
	pub syntax: SyntaxDetails,
	/// Details about the email verification used for debugging.
	pub debug: DebugDetails,
}

impl Default for CheckEmailOutput {
	fn default() -> Self {
		CheckEmailOutput {
			input: String::default(),
			is_reachable: Reachable::Unknown,
			misc: Ok(MiscDetails::default()),
			mx: Ok(MxDetails::default()),
			smtp: Ok(SmtpDetails::default()),
			syntax: SyntaxDetails::default(),
			debug: DebugDetails::default(),
		}
	}
}

// Implement a custom serialize.
impl Serialize for CheckEmailOutput {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// This is just used internally to get the nested error field.
		#[derive(Serialize)]
		struct MyError<E> {
			error: E,
			// We add an optional "description" field when relevant, given by
			// the `get_description` on SmtpError.
			#[serde(skip_serializing_if = "Option::is_none")]
			description: Option<SmtpErrorDesc>,
		}

		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry("input", &self.input)?;
		map.serialize_entry("is_reachable", &self.is_reachable)?;
		match &self.misc {
			Ok(t) => map.serialize_entry("misc", &t)?,
			Err(error) => map.serialize_entry(
				"misc",
				&MyError {
					error,
					description: None,
				},
			)?,
		}
		match &self.mx {
			Ok(t) => map.serialize_entry("mx", &t)?,
			Err(error) => map.serialize_entry(
				"mx",
				&MyError {
					error,
					description: None,
				},
			)?,
		}
		match &self.smtp {
			Ok(t) => map.serialize_entry("smtp", &t)?,
			Err(error) => map.serialize_entry(
				"smtp",
				&MyError {
					error,
					description: error.get_description(),
				},
			)?,
		}
		map.serialize_entry("syntax", &self.syntax)?;
		map.serialize_entry("debug", &self.debug)?;
		map.end()
	}
}

#[cfg(test)]
mod tests {
	use super::{CheckEmailOutput, DebugDetails};
	use async_smtp::smtp::response::{Category, Code, Detail, Response, Severity};

	#[test]
	fn should_serialize_correctly() {
		// create a dummy CheckEmailOutput, with a given message as a transient
		// SMTP error.
		fn dummy_response_with_message(m: &str) -> CheckEmailOutput {
			let r = Response::new(
				Code {
					severity: Severity::TransientNegativeCompletion,
					category: Category::MailSystem,
					detail: Detail::Zero,
				},
				vec![m.to_string(), "8BITMIME".to_string(), "SIZE 42".to_string()],
			);

			CheckEmailOutput {
				input: "foo".to_string(),
				is_reachable: super::Reachable::Unknown,
				misc: Ok(super::MiscDetails::default()),
				mx: Ok(super::MxDetails::default()),
				syntax: super::SyntaxDetails::default(),
				smtp: Err(super::SmtpError::SmtpError(r.into())),
				debug: DebugDetails::default(),
			}
		}

		let res = dummy_response_with_message("blacklist");
		let actual = serde_json::to_string(&res).unwrap();
		// Make sure the `description` is present with IpBlacklisted.
		let expected = r#""smtp":{"error":{"type":"SmtpError","message":"transient: blacklist"},"description":"IpBlacklisted"}"#;
		assert!(actual.contains(expected));

		let res =
			dummy_response_with_message("Client host rejected: cannot find your reverse hostname");
		let actual = serde_json::to_string(&res).unwrap();
		// Make sure the `description` is present with NeedsRDNs.
		let expected = r#"smtp":{"error":{"type":"SmtpError","message":"transient: Client host rejected: cannot find your reverse hostname"},"description":"NeedsRDNS"}"#;
		assert!(actual.contains(expected));

		let res = dummy_response_with_message("foobar");
		let actual = serde_json::to_string(&res).unwrap();
		// Make sure the `description` is NOT present.
		let expected = r#""smtp":{"error":{"type":"SmtpError","message":"transient: foobar"}}"#;
		assert!(actual.contains(expected));
	}
}
