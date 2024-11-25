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

use super::gmail::GmailError;

use super::headless::HeadlessError;
use super::outlook::microsoft365::Microsoft365Error;
use super::parser;
use super::yahoo::YahooError;
use crate::util::ser_with_display::ser_with_display;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use fast_socks5::SocksError;
use serde::Serialize;
use thiserror::Error;

/// Error occured connecting to this email server via SMTP.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum SmtpError {
	/// Error if we're using a SOCKS5 proxy.
	#[serde(serialize_with = "ser_with_display")]
	#[error("SOCKS5 error: {0}")]
	SocksError(SocksError),
	/// Error when communicating with SMTP server.
	#[serde(serialize_with = "ser_with_display")]
	#[error("SMTP error: {0}")]
	SmtpError(AsyncSmtpError),
	/// Error when verifying a Yahoo email via HTTP requests.
	#[error("Yahoo error: {0}")]
	YahooError(YahooError),
	/// Error when verifying a Gmail email via a HTTP request.
	#[error("Gmail error: {0}")]
	GmailError(GmailError),
	/// Error when verifying a Hotmail email via headless browser.
	#[error("Headless verification error: {0}")]
	HeadlessError(HeadlessError),
	/// Error when verifying a Microsoft 365 email via HTTP request.
	#[error("Microsoft 365 API error: {0}")]
	Microsoft365Error(Microsoft365Error),
}

impl From<SocksError> for SmtpError {
	fn from(e: SocksError) -> Self {
		SmtpError::SocksError(e)
	}
}

impl From<YahooError> for SmtpError {
	fn from(e: YahooError) -> Self {
		SmtpError::YahooError(e)
	}
}

impl From<GmailError> for SmtpError {
	fn from(e: GmailError) -> Self {
		SmtpError::GmailError(e)
	}
}

impl From<HeadlessError> for SmtpError {
	fn from(e: HeadlessError) -> Self {
		SmtpError::HeadlessError(e)
	}
}

impl From<Microsoft365Error> for SmtpError {
	fn from(e: Microsoft365Error) -> Self {
		SmtpError::Microsoft365Error(e)
	}
}

impl SmtpError {
	/// Get a human-understandable description of the error, in form of an enum
	/// SmtpErrorDesc. This only parses the following known errors:
	/// - IP blacklisted
	/// - IP needs reverse DNS
	pub fn get_description(&self) -> Option<SmtpErrorDesc> {
		match self {
			SmtpError::SmtpError(_) => {
				if parser::is_err_ip_blacklisted(self) {
					Some(SmtpErrorDesc::IpBlacklisted)
				} else if parser::is_err_needs_rdns(self) {
					Some(SmtpErrorDesc::NeedsRDNS)
				} else {
					None
				}
			}
			_ => None,
		}
	}
}

#[derive(Debug, Serialize)]
/// SmtpErrorDesc describes a description of which category the error belongs
/// to.
pub enum SmtpErrorDesc {
	/// The IP is blacklisted.
	IpBlacklisted,
	/// The IP needs a reverse DNS entry.
	NeedsRDNS,
}
