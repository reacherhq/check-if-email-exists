// check-if-email-exists
// Copyright (C) 2018-2020 Amaury Martiny

// check-if-email-exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check-if-email-exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check-if-email-exists.  If not, see <http://www.gnu.org/licenses/>.

extern crate lettre;
#[macro_use]
extern crate log;
extern crate mailchecker;
extern crate native_tls;
extern crate rand;
extern crate serde;
extern crate trust_dns_resolver;

pub mod misc;
pub mod mx;
pub mod smtp;
pub mod syntax;
mod util;

use futures::future::select_ok;
use lettre::{smtp::SMTP_PORT, EmailAddress};
use misc::{misc_details, MiscDetails, MiscError};
use mx::{get_mx_lookup, MxDetails, MxError};
use serde::{ser::SerializeMap, Serialize, Serializer};
use smtp::{SmtpDetails, SmtpError};
use std::str::FromStr;
use syntax::{address_syntax, SyntaxDetails, SyntaxError};

/// All details about email address, MX records and SMTP responses
#[derive(Debug)]
pub struct SingleEmail {
	/// Input by the user
	pub input: String,
	/// Misc details about the email address
	pub misc: Result<MiscDetails, MiscError>,
	/// Details about the MX host
	pub mx: Result<MxDetails, MxError>,
	/// Details about the SMTP responses of the email
	pub smtp: Result<SmtpDetails, SmtpError>, // TODO Better Err type
	/// Details about the email address
	pub syntax: Result<SyntaxDetails, SyntaxError>,
}

// Implement a custom serialize
impl Serialize for SingleEmail {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// This is just used internally to get the nested error field
		#[derive(Serialize)]
		struct MyError<E> {
			error: E,
		}

		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry("input", &self.input)?;
		match &self.misc {
			Ok(t) => map.serialize_entry("misc", &t)?,
			Err(error) => map.serialize_entry("misc", &MyError { error })?,
		}
		match &self.mx {
			Ok(t) => map.serialize_entry("mx", &t)?,
			Err(error) => map.serialize_entry("mx", &MyError { error })?,
		}
		match &self.smtp {
			Ok(t) => map.serialize_entry("smtp", &t)?,
			Err(error) => map.serialize_entry("smtp", &MyError { error })?,
		}
		match &self.syntax {
			Ok(t) => map.serialize_entry("syntax", &t)?,
			Err(error) => map.serialize_entry("syntax", &MyError { error })?,
		}
		map.end()
	}
}

/// The main function: checks email format, checks MX records, and checks SMTP
/// responses to the email inbox.
pub async fn email_exists(to_email: &str, from_email: &str) -> SingleEmail {
	let from_email = EmailAddress::from_str(from_email).unwrap_or_else(|_| {
		EmailAddress::from_str("user@example.org").expect("This is a valid email. qed.")
	});

	debug!("Checking email '{}'", to_email);
	let my_syntax = match address_syntax(to_email) {
		Ok(s) => s,
		e => {
			return SingleEmail {
				input: to_email.into(),
				misc: Err(MiscError::Skipped),
				mx: Err(MxError::Skipped),
				smtp: Err(SmtpError::Skipped),
				syntax: e,
			}
		}
	};
	debug!("Details of the email address: {:?}", my_syntax);

	debug!("Getting MX lookup...");
	let my_mx = match get_mx_lookup(&my_syntax) {
		Ok(m) => m,
		e => {
			return SingleEmail {
				input: to_email.into(),
				misc: Err(MiscError::Skipped),
				mx: e,
				smtp: Err(SmtpError::Skipped),
				syntax: Ok(my_syntax),
			}
		}
	};
	debug!("Found the following MX hosts {:?}", my_mx);

	debug!("Getting misc details...");
	let my_misc = misc_details(&my_syntax);

	// Create one future per lookup result
	let futures = my_mx
		.lookup
		.iter()
		.map(|host| {
			let fut = smtp::smtp_details(
				&from_email,
				&my_syntax.address,
				host.exchange(),
				// We could add ports 465 and 587 too
				SMTP_PORT,
				my_syntax.domain.as_str(),
			);

			// https://rust-lang.github.io/async-book/04_pinning/01_chapter.html
			Box::pin(fut)
		})
		.collect::<Vec<_>>();

	// Race, return the first future that resolves
	let my_smtp = match select_ok(futures).await {
		Ok((details, _)) => Ok(details),
		Err(err) => Err(err),
	};

	SingleEmail {
		input: to_email.into(),
		misc: Ok(my_misc),
		mx: Ok(my_mx),
		smtp: my_smtp,
		syntax: Ok(my_syntax),
	}
}
