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

extern crate lettre;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate rand;
extern crate rayon;
extern crate trust_dns_resolver;

mod mx_hosts;
mod smtp;

use lettre::smtp::SMTP_PORT;
use lettre::EmailAddress;
use mx_hosts::MxLookupError;
use rayon::prelude::*;
use smtp::EmailDetails;
use std::io::Error as IoError;
use trust_dns_resolver::error::ResolveError;

/// Errors that are returned by email_exists
#[derive(Debug)]
pub enum EmailExistsError {
	BlockedByIsp,           // ISP is blocking SMTP ports
	Io(IoError),            // IO error
	MxLookup(ResolveError), // Error while resolving MX lookups
}

pub fn email_exists(
	from_email: &EmailAddress,
	to_email: &EmailAddress,
) -> Result<EmailDetails, EmailExistsError> {
	debug!("Checking email '{}'", to_email);

	let domain = to_email.to_string();
	let domain = domain
		.as_str()
		.split("@")
		.skip(1)
		.next()
		.expect("We checked above that email is valid. qed.");
	debug!("Domain name is '{}'", domain);

	debug!("Getting MX lookup...");
	let hosts = match mx_hosts::get_mx_lookup(domain) {
		Ok(h) => h,
		Err(MxLookupError::Io(err)) => {
			return Err(EmailExistsError::Io(err));
		}
		Err(MxLookupError::ResolveError(err)) => {
			return Err(EmailExistsError::MxLookup(err));
		}
	};

	let mut combinations = Vec::new(); // `(host, port)` combination
	for host in hosts.iter() {
		// We could add ports 465 and 587 too
		combinations.push((host.exchange(), SMTP_PORT));
	}
	debug!("Found the following MX hosts {:?}", combinations);

	combinations
		// Concurrently find any combination that returns true for email_exists
		.par_iter()
		// Attempt to make a SMTP call to host
		.flat_map(|(host, port)| smtp::email_details(from_email, to_email, host, *port, domain))
		.find_any(|_| true)
		// If all smtp calls timed out/got refused/errored, we assume that the
		// ISP is blocking relevant ports
		.ok_or(EmailExistsError::BlockedByIsp)
}
