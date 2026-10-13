use crate::syntax::SyntaxDetails;
use crate::util::ser_with_display::ser_with_display;
use hickory_resolver::error::{ResolveError, ResolveErrorKind};
use hickory_resolver::lookup::MxLookup;
use hickory_resolver::system_conf::read_system_conf;
use hickory_resolver::TokioAsyncResolver;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::io;
use thiserror::Error;

/// Details about the MX lookup.
#[derive(Debug)]
pub struct MxDetails {
	/// MX lookup of this DNS.
	pub lookup: Result<MxLookup, ResolveError>,
}

impl Default for MxDetails {
	fn default() -> Self {
		MxDetails {
			lookup: Err(ResolveError::from("Skipped")),
		}
	}
}

impl From<MxLookup> for MxDetails {
	fn from(lookup: MxLookup) -> Self {
		MxDetails { lookup: Ok(lookup) }
	}
}

impl Serialize for MxDetails {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let records: Vec<String> = self
			.lookup
			.as_ref()
			.map(|lookup| {
				lookup
					.iter()
					.map(|host| host.exchange().to_string())
					.collect::<Vec<_>>()
			})
			.unwrap_or_else(|_| Vec::new()); // In case of a resolve error, we don't serialize the error.

		let mut map = serializer.serialize_map(Some(2))?;
		map.serialize_entry("accepts_mail", &!records.is_empty())?;
		map.serialize_entry("records", &records)?;
		map.end()
	}
}

/// Errors that can happen on MX lookups.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum MxError {
	/// Error with IO.
	#[serde(serialize_with = "ser_with_display")]
	#[error("IO error: {0}")]
	IoError(io::Error),
	/// Error while resolving MX lookups.
	#[serde(serialize_with = "ser_with_display")]
	#[error("Resolve error: {0}")]
	ResolveError(Box<ResolveError>),
}

impl From<io::Error> for MxError {
	fn from(e: io::Error) -> Self {
		MxError::IoError(e)
	}
}

impl From<ResolveError> for MxError {
	fn from(e: ResolveError) -> Self {
		MxError::ResolveError(Box::new(e))
	}
}

/// Make a MX lookup.
pub async fn check_mx(syntax: &SyntaxDetails) -> Result<MxDetails, MxError> {
	// Construct a new Resolver with default configuration options
	let (config, opts) = read_system_conf()?;
	let resolver = TokioAsyncResolver::tokio(config, opts);

	match resolver.mx_lookup(&syntax.domain).await {
		Ok(lookup) => Ok(MxDetails::from(lookup)),
		Err(err) => match err.kind() {
			// Prefer to return an empty MX lookup if there are no records.
			ResolveErrorKind::NoRecordsFound { .. } => Ok(MxDetails { lookup: Err(err) }),
			_ => Err(err.into()),
		},
	}
}

/// Check if the MX host is from Google, i.e. either a @gmail.com address, or
/// a Google Suite email.
pub fn is_gmail(mx_host: &str) -> bool {
	mx_host.to_lowercase().ends_with(".google.com.")
}

/// Check if a MX host is from outlook (includes @hotmail.*, @outlook.* and
/// all Microsoft 365 addresses).
///
/// After some testing, I got:
/// - *@outlook.com -> `outlook-com.olc.protection.outlook.com.`
/// - *@outlook.fr -> `eur.olc.protection.outlook.com.`
/// - *@hotmail.com -> `hotmail-com.olc.protection.outlook.com.`
/// - *@hotmail.fr -> `eur.olc.protection.outlook.com.`
/// - *@hotmail.nl -> `eur.olc.protection.outlook.com.`
///
/// But Microsoft 365 (B2B) addresses end with "mail.protection.outlook.com."
///
/// Microsoft is also migrating domains to MX hosts under "mx.microsoft.",
/// both for consumer domains and Microsoft 365 tenants:
/// - *@outlook.cz -> `outlook-cz.d-v1.mx.microsoft.`
/// - *@hotmail.cz -> `hotmail-cz.f-v1.mx.microsoft.`
/// - *@contoso.com -> `contoso-com.a-v1.mx.microsoft.`
///
/// TL;DR:
/// - B2C emails -> end with ".olc.protection.outlook.com.", or a consumer
///   domain under ".mx.microsoft." (see `is_hotmail_b2c`)
/// - B2B emails -> everything else
pub fn is_hotmail(mx_host: &str) -> bool {
	let host = mx_host.to_lowercase();
	host.ends_with(".protection.outlook.com.") || host.ends_with(".mx.microsoft.")
}

/// Check if an address is a Hotmail B2B email address.
pub fn is_hotmail_b2b(mx_host: &str) -> bool {
	is_hotmail(mx_host) && !is_hotmail_b2c(mx_host)
}

/// Check if an address is a Hotmail B2C email address.
pub fn is_hotmail_b2c(mx_host: &str) -> bool {
	let host = mx_host.to_lowercase();
	if host.ends_with(".olc.protection.outlook.com.") {
		return true;
	}

	// On "mx.microsoft.", consumer and Microsoft 365 domains share the same
	// MX format, with the first label being the domain with dots replaced by
	// dashes. We consider it B2C if that label looks like a Microsoft consumer
	// domain, i.e. a consumer brand followed by a TLD (e.g. "outlook-cz" or
	// "hotmail-co-uk").
	let Some(rest) = host.strip_suffix(".mx.microsoft.") else {
		return false;
	};
	let label = rest.split('.').next().unwrap_or_default();
	let mut parts = label.split('-');
	let is_consumer_brand = matches!(
		parts.next(),
		Some("hotmail" | "outlook" | "live" | "msn" | "windowslive")
	);
	let tld_parts: Vec<&str> = parts.collect();

	is_consumer_brand
		&& !tld_parts.is_empty()
		&& tld_parts
			.iter()
			.all(|p| (2..=3).contains(&p.len()) && p.chars().all(|c| c.is_ascii_alphabetic()))
}

/// Check if the MX host is behind Mimecast.
pub fn is_mimecast(mx_host: &str) -> bool {
	mx_host.to_lowercase().ends_with(".mimecast.com.")
}

/// Check if the MX host is behind Proofpoint.
pub fn is_proofpoint(mx_host: &str) -> bool {
	let host = mx_host.to_lowercase();
	host.ends_with(".pphosted.com.") || host.ends_with("ppe-hosted.com.")
}

/// Check if the MX host is from Yahoo.
/// Examples:
/// - mta7.am0.yahoodns.net.
/// - mx-eu.mail.am0.yahoodns.net.
pub fn is_yahoo(mx_host: &str) -> bool {
	mx_host.to_lowercase().ends_with(".yahoodns.net.")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn proofpoint_detection_is_case_insensitive() {
		assert!(is_proofpoint("mx1.PPHOSTED.COM."));
		assert!(is_proofpoint("mx2.ppe-hosted.com."));
	}

	#[test]
	fn hotmail_b2c_detection() {
		for host in [
			"outlook-com.olc.protection.outlook.com.",
			"eur.olc.protection.outlook.com.",
			// https://github.com/reacherhq/check-if-email-exists/issues/1663
			"outlook-cz.d-v1.mx.microsoft.",
			"hotmail-cz.f-v1.mx.microsoft.",
			"HOTMAIL-CO-UK.a-v1.mx.microsoft.",
			"live-com.b-v1.mx.microsoft.",
		] {
			assert!(is_hotmail(host), "{}", host);
			assert!(is_hotmail_b2c(host), "{}", host);
			assert!(!is_hotmail_b2b(host), "{}", host);
		}
	}

	#[test]
	fn hotmail_b2b_detection() {
		for host in [
			"contoso-com.mail.protection.outlook.com.",
			"contoso-com.a-v1.mx.microsoft.",
			"outlook-tips-blog.a-v1.mx.microsoft.",
			"hotmail.a-v1.mx.microsoft.",
		] {
			assert!(is_hotmail(host), "{}", host);
			assert!(!is_hotmail_b2c(host), "{}", host);
			assert!(is_hotmail_b2b(host), "{}", host);
		}
	}

	#[test]
	fn non_microsoft_hosts() {
		for host in ["gmail-smtp-in.l.google.com.", "mx.microsoft.example.com."] {
			assert!(!is_hotmail(host), "{}", host);
			assert!(!is_hotmail_b2c(host), "{}", host);
			assert!(!is_hotmail_b2b(host), "{}", host);
		}
	}
}
