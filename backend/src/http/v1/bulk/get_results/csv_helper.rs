// Reacher - Email Verification
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

use serde::Serialize;
use std::convert::TryFrom;

/// Wrapper for serde json value to convert
/// into a csv response
#[derive(Debug)]
pub struct CsvWrapper(pub serde_json::Value);

/// Simplified output of `CheckEmailOutput` struct
/// for csv fields.
#[derive(Debug, Serialize)]
pub struct CsvResponse {
	input: String,
	is_reachable: String,
	#[serde(rename = "misc.is_disposable")]
	misc_is_disposable: bool,
	#[serde(rename = "misc.is_role_account")]
	misc_is_role_account: bool,
	#[serde(rename = "misc.gravatar_url")]
	misc_gravatar_url: Option<String>,
	#[serde(rename = "mx.accepts_mail")]
	mx_accepts_mail: bool,
	#[serde(rename = "smtp.can_connect")]
	smtp_can_connect: bool,
	#[serde(rename = "smtp.has_full_inbox")]
	smtp_has_full_inbox: bool,
	#[serde(rename = "smtp.is_catch_all")]
	smtp_is_catch_all: bool,
	#[serde(rename = "smtp.is_deliverable")]
	smtp_is_deliverable: bool,
	#[serde(rename = "smtp.is_disabled")]
	smtp_is_disabled: bool,
	#[serde(rename = "syntax.is_valid_syntax")]
	syntax_is_valid_syntax: bool,
	#[serde(rename = "syntax.domain")]
	syntax_domain: String,
	#[serde(rename = "syntax.username")]
	syntax_username: String,
	error: Option<String>,
}

impl TryFrom<CsvWrapper> for CsvResponse {
	type Error = &'static str;

	fn try_from(value: CsvWrapper) -> Result<Self, Self::Error> {
		let top_level = value
			.0
			.as_object()
			.ok_or("Failed to find top level object")?;

		let input = top_level
			.get("input")
			.and_then(|v| v.as_str())
			.ok_or("input should be a string")?
			.to_string();
		let is_reachable = top_level
			.get("is_reachable")
			.and_then(|v| v.as_str())
			.ok_or("is_reachable should be a string")?
			.to_string();

		let misc = top_level
			.get("misc")
			.and_then(|v| v.as_object())
			.ok_or("misc field should be an object")?;
		let misc_is_disposable = misc
			.get("is_disposable")
			.and_then(|v| v.as_bool())
			.ok_or("is_disposable should be a boolean")?;
		let misc_is_role_account = misc
			.get("is_role_account")
			.and_then(|v| v.as_bool())
			.ok_or("is_role_account should be a boolean")?;
		let misc_gravatar_url = misc
			.get("gravatar_url")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string());

		let mx = top_level
			.get("mx")
			.and_then(|v| v.as_object())
			.ok_or("mx field should be an object")?;
		let mx_accepts_mail = mx
			.get("accepts_email")
			.and_then(|v| v.as_bool())
			.ok_or("accepts_email should be a boolean")?;

		let smtp = top_level
			.get("smtp")
			.and_then(|v| v.as_object())
			.ok_or("smtp field should be an object")?;
		let smtp_can_connect = smtp
			.get("can_connect_smtp")
			.and_then(|v| v.as_bool())
			.ok_or("can_connect_smtp should be a boolean")?;
		let smtp_has_full_inbox = smtp
			.get("has_full_inbox")
			.and_then(|v| v.as_bool())
			.ok_or("has_full_inbox should be a boolean")?;
		let smtp_is_catch_all = smtp
			.get("is_catch_all")
			.and_then(|v| v.as_bool())
			.ok_or("is_catch_all should be a boolean")?;
		let smtp_is_deliverable = smtp
			.get("is_deliverable")
			.and_then(|v| v.as_bool())
			.ok_or("is_deliverable should be a boolean")?;
		let smtp_is_disabled = smtp
			.get("is_disabled")
			.and_then(|v| v.as_bool())
			.ok_or("is_disabled should be a boolean")?;

		let syntax = top_level
			.get("syntax")
			.and_then(|v| v.as_object())
			.ok_or("syntax field should be an object")?;
		let syntax_is_valid_syntax = syntax
			.get("is_valid_syntax")
			.and_then(|v| v.as_bool())
			.ok_or("is_valid_syntax should be a boolean")?;
		let syntax_domain = syntax
			.get("domain")
			.and_then(|v| v.as_str())
			.ok_or("domain should be a string")?
			.to_string();
		let syntax_username = syntax
			.get("username")
			.and_then(|v| v.as_str())
			.ok_or("username should be a string")?
			.to_string();

		let error = top_level.get("error").map(|v| v.to_string());

		Ok(CsvResponse {
			input,
			is_reachable,
			misc_is_disposable,
			misc_is_role_account,
			misc_gravatar_url,
			mx_accepts_mail,
			smtp_can_connect,
			smtp_has_full_inbox,
			smtp_is_catch_all,
			smtp_is_deliverable,
			smtp_is_disabled,
			syntax_is_valid_syntax,
			syntax_domain,
			syntax_username,
			error,
		})
	}
}
