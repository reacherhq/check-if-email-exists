// check_if_email_exists
// Copyright (C) 2018-2019 Amaury Martiny

// check_if_email_exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check_if_email_exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check_if_email_exists.  If not, see <http://www.gnu.org/licenses/>.

use crate::util::ser_with_display;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::io::Error;
use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::lookup::MxLookup;
use trust_dns_resolver::Resolver;

/// Details about the MX lookup
#[derive(Debug)]
pub struct MxDetails(pub MxLookup);

impl Serialize for MxDetails {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry(
			"records",
			&self
				.0
				.iter()
				.map(|host| host.exchange().to_string())
				.collect::<Vec<_>>(),
		)?;
		map.end()
	}
}

/// Errors that can happen on MX lookups
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum MxError {
	/// Skipped checking MX records
	Skipped,
	/// Error with IO
	#[serde(serialize_with = "ser_with_display")]
	IoError(Error),
	/// Error while resolving MX lookups
	#[serde(serialize_with = "ser_with_display")]
	ResolveError(ResolveError),
}

/// Make a MX lookup
pub fn get_mx_lookup(domain: &str) -> Result<MxDetails, MxError> {
	// Construct a new Resolver with default configuration options
	let resolver = match Resolver::new(ResolverConfig::default(), ResolverOpts::default()) {
		Ok(r) => r,
		Err(err) => {
			return Err(MxError::IoError(err));
		}
	};

	// Lookup the MX records associated with a name.
	// The final dot forces this to be an FQDN, otherwise the search rules as specified
	// in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
	match resolver.mx_lookup(domain) {
		Ok(lookup) => Ok(MxDetails(lookup)),
		Err(err) => Err(MxError::ResolveError(err)),
	}
}
