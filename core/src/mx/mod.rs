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
