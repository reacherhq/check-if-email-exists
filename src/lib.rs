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
extern crate serde;
extern crate trust_dns_resolver;

mod mx;
mod smtp;
mod syntax;
mod util;

use lettre::smtp::SMTP_PORT;
use lettre::EmailAddress;
use mx::{get_mx_lookup, MxDetails, MxError};
use rayon::prelude::*;
use serde::Serialize;
use smtp::{SmtpDetails, SmtpError};
use std::str::FromStr;
use syntax::{address_syntax, SyntaxDetails, SyntaxError};

/// All details about email address, MX records and SMTP responses
#[derive(Debug, Serialize)]
pub struct SingleEmail {
	/// Details about the MX host
	pub mx: Result<MxDetails, MxError>,
	/// Details about the SMTP responses of the email
	pub smtp: Result<SmtpDetails, SmtpError>, // TODO Better Err type
	/// Details about the email address
	pub syntax: Result<SyntaxDetails, SyntaxError>,
}

/// The main function: checks email format, checks MX records, and checks SMTP
/// responses to the email inbox.
pub fn email_exists(to_email: &str, from_email: &str) -> SingleEmail {
	let from_email = EmailAddress::from_str(from_email).unwrap_or(
		EmailAddress::from_str("user@example.org").expect("This is a valid email. qed."),
	);

	debug!("Checking email '{}'", to_email);
	let my_syntax = match address_syntax(to_email) {
		Ok(s) => s,
		e => {
			return SingleEmail {
				mx: Err(MxError::Skipped),
				smtp: Err(SmtpError::Skipped),
				syntax: e,
			}
		}
	};
	debug!("Details of the email address: {:?}", my_syntax);

	debug!("Getting MX lookup...");
	let my_mx = match get_mx_lookup(&my_syntax.domain) {
		Ok(m) => m,
		e => {
			return SingleEmail {
				mx: e,
				smtp: Err(SmtpError::Skipped),
				syntax: Ok(my_syntax),
			}
		}
	};
	debug!("Found the following MX hosts {:?}", my_mx);

	// `(host, port)` combination
	// We could add ports 465 and 587 too
	let combinations = my_mx
		.lookup
		.iter()
		.map(|host| (host.exchange(), SMTP_PORT))
		.collect::<Vec<_>>();

	let my_smtp = combinations
		// Concurrently find any combination that returns true for email_exists
		.par_iter()
		// Attempt to make a SMTP call to host
		.flat_map(|(host, port)| {
			smtp::smtp_details(
				&from_email,
				&my_syntax.address,
				host,
				*port,
				my_syntax.domain.as_str(),
			)
		})
		.find_any(|_| true)
		// If all smtp calls timed out/got refused/errored, we assume that the
		// ISP is blocking relevant ports
		.ok_or(SmtpError::BlockedByIsp);

	SingleEmail {
		mx: Ok(my_mx),
		smtp: my_smtp,
		syntax: Ok(my_syntax),
	}
}
