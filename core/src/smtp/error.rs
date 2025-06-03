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
use async_smtp::error::Error as AsyncSmtpError;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;

/// Error occurred connecting to this email server via SMTP.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum SmtpError {
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
	/// Error from async-smtp crate.
	#[error("SMTP error: {0}")]
	#[serde(serialize_with = "ser_with_display")]
	AsyncSmtpError(AsyncSmtpError),
	/// I/O error.
	#[error("I/O error: {0}")]
	#[serde(serialize_with = "ser_with_display")]
	IOError(std::io::Error),
	/// Timeout error.
	#[error("Timeout error: {0:?}")]
	Timeout(Duration),
	/// SOCKS5 proxy error.
	#[error("SOCKS5 error: {0}")]
	#[serde(serialize_with = "ser_with_display")]
	Socks5(fast_socks5::SocksError),
	/// Anyhow error.
	/// This is a catch-all error type for any error that can't be categorized
	/// into the above types.
	#[error("Anyhow error: {0}")]
	#[serde(serialize_with = "ser_with_display")]
	AnyhowError(anyhow::Error),
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

impl From<AsyncSmtpError> for SmtpError {
	fn from(e: AsyncSmtpError) -> Self {
		SmtpError::AsyncSmtpError(e)
	}
}

impl From<std::io::Error> for SmtpError {
	fn from(e: std::io::Error) -> Self {
		SmtpError::IOError(e)
	}
}

impl From<fast_socks5::SocksError> for SmtpError {
	fn from(e: fast_socks5::SocksError) -> Self {
		SmtpError::Socks5(e)
	}
}

impl From<anyhow::Error> for SmtpError {
	fn from(e: anyhow::Error) -> Self {
		SmtpError::AnyhowError(e)
	}
}

impl SmtpError {
	/// Get a human-understandable description of the error, in form of an enum
	/// SmtpErrorDesc. This only parses the following known errors:
	/// - IP blacklisted
	/// - IP needs reverse DNS
	pub fn get_description(&self) -> Option<SmtpErrorDesc> {
		match self {
			SmtpError::AsyncSmtpError(_) => {
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
