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

mod api;
#[cfg(feature = "headless")]
mod headless;

use crate::util::ser_with_display::ser_with_display;
use reqwest::Error as ReqwestError;
use serde::Serialize;
use serde_json::error::Error as SerdeError;
use std::fmt;

pub use api::check_api;
#[cfg(feature = "headless")]
pub use headless::check_headless;

/// Possible errors when checking Yahoo email addresses.
#[derive(Debug, Serialize)]
pub enum YahooError {
	/// Cannot find "acrumb" field in cookie.
	NoAcrumb,
	/// Cannot find "sessionIndex" hidden input in body
	NoSessionIndex,
	/// Cannot find cookie in Yahoo response.
	NoCookie,
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	ReqwestError(ReqwestError),
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	SerdeError(SerdeError),
}

impl fmt::Display for YahooError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// Customize so only `x` and `y` are denoted.
		write!(f, "{self:?}")
	}
}

impl From<ReqwestError> for YahooError {
	fn from(error: ReqwestError) -> Self {
		YahooError::ReqwestError(error)
	}
}

impl From<SerdeError> for YahooError {
	fn from(error: SerdeError) -> Self {
		YahooError::SerdeError(error)
	}
}

/// Check if the MX host is from Yahoo.
/// Examples:
/// - mta7.am0.yahoodns.net.
/// - mx-eu.mail.am0.yahoodns.net.
pub fn is_yahoo(host: &str) -> bool {
	host.to_lowercase().ends_with(".yahoodns.net.")
}
