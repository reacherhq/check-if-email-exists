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
use serde::{Serialize,Serializer};
use std::io::Error;
use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::lookup::MxLookup;
use trust_dns_resolver::Resolver;

/// Custom implementation of `Serialize` for a `MxLookup`
pub fn ser_with_mx_lookup<S>(lookup: &MxLookup, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.collect_seq(lookup.iter().map(|host| host.exchange().to_string()))
}

/// Details about the MX lookup
#[derive(Debug, Serialize)]
pub struct MxDetails {
	#[serde(serialize_with = "ser_with_mx_lookup")]
	pub lookup: MxLookup,
}

/// Errors that can happen on MX lookups
#[derive(Debug, Serialize)]
pub enum MxError {
	/// Skipped checking MX records
	Skipped,
	/// Error with IO
	#[serde(serialize_with = "ser_with_display")]
	Io(Error),
	/// Error while resolving MX lookups
	#[serde(serialize_with = "ser_with_display")]
	Resolve(ResolveError),
}

pub fn get_mx_lookup(domain: &str) -> Result<MxDetails, MxError> {
	// Construct a new Resolver with default configuration options
	let resolver = match Resolver::new(ResolverConfig::default(), ResolverOpts::default()) {
		Ok(r) => r,
		Err(err) => {
			return Err(MxError::Io(err));
		}
	};

	// Lookup the MX records associated with a name.
	// The final dot forces this to be an FQDN, otherwise the search rules as specified
	// in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
	match resolver.mx_lookup(domain) {
		Ok(lookup) => Ok(MxDetails { lookup }),
		Err(err) => Err(MxError::Resolve(err)),
	}
}
