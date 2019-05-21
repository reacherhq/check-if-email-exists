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

use std::io::Error as IoError;
use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::lookup::MxLookup;
use trust_dns_resolver::Resolver;

pub enum MxLookupError {
	Io(IoError),
	ResolveError(ResolveError),
}

pub fn get_mx_lookup(domain: &str) -> Result<MxLookup, MxLookupError> {
	// Construct a new Resolver with default configuration options
	let resolver = match Resolver::new(ResolverConfig::default(), ResolverOpts::default()) {
		Ok(r) => r,
		Err(err) => {
			return Err(MxLookupError::Io(err));
		}
	};

	// Lookup the MX records associated with a name.
	// The final dot forces this to be an FQDN, otherwise the search rules as specified
	// in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
	match resolver.mx_lookup(domain) {
		Ok(l) => Ok(l),
		Err(err) => Err(MxLookupError::ResolveError(err)),
	}
}
