// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

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
pub struct JobResultCsvResponse {
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

/// Convert csv wrapper to csv response
/// Performs multiple allocations for string fields
/// throw error if field is missing
impl TryFrom<CsvWrapper> for JobResultCsvResponse {
	type Error = &'static str;

	fn try_from(value: CsvWrapper) -> Result<Self, Self::Error> {
		let mut input: String = String::default();
		let mut is_reachable: String = String::default();
		let mut misc_is_disposable: bool = false;
		let mut misc_is_role_account: bool = false;
		let mut misc_gravatar_url: Option<String> = None;
		let mut mx_accepts_mail: bool = false;
		let mut smtp_can_connect: bool = false;
		let mut smtp_has_full_inbox: bool = false;
		let mut smtp_is_catch_all: bool = false;
		let mut smtp_is_deliverable: bool = false;
		let mut smtp_is_disabled: bool = false;
		let mut syntax_is_valid_syntax: bool = false;
		let mut syntax_domain: String = String::default();
		let mut syntax_username: String = String::default();
		let mut error: Option<String> = None;

		let top_level = value
			.0
			.as_object()
			.ok_or("Failed to find top level object")?;
		for (key, val) in top_level.keys().zip(top_level.values()) {
			match key.as_str() {
				"input" => input = val.as_str().ok_or("input should be a string")?.to_string(),
				"is_reachable" => {
					is_reachable = val
						.as_str()
						.ok_or("is_reachable should be a string")?
						.to_string()
				}
				"misc" => {
					let misc_obj = val.as_object().ok_or("misc field should be an object")?;
					for (key, val) in misc_obj.keys().zip(misc_obj.values()) {
						match key.as_str() {
							"error" => error = Some(val.to_string()),
							"is_disposable" => {
								misc_is_disposable =
									val.as_bool().ok_or("is_disposable should be a boolean")?
							}
							"is_role_account" => {
								misc_is_role_account =
									val.as_bool().ok_or("is_role_account should be a boolean")?
							}
							"gravatar_url" => {
								if Option::is_some(&val.as_str()) {
									misc_gravatar_url = Some(val.to_string())
								}
							}
							_ => {}
						}
					}
				}
				"mx" => {
					let mx_obj = val.as_object().ok_or("mx field should be an object")?;
					for (key, val) in mx_obj.keys().zip(mx_obj.values()) {
						match key.as_str() {
							"error" => error = Some(val.to_string()),
							"accepts_email" => {
								mx_accepts_mail =
									val.as_bool().ok_or("accepts_email should be a boolean")?
							}
							_ => {}
						}
					}
				}
				"smtp" => {
					let smtp_obj = val.as_object().ok_or("mx field should be an object")?;
					for (key, val) in smtp_obj.keys().zip(smtp_obj.values()) {
						match key.as_str() {
							"error" => error = Some(val.to_string()),
							"can_connect_smtp" => {
								smtp_can_connect = val
									.as_bool()
									.ok_or("can_connect_smtp should be a boolean")?
							}
							"has_full_inbox" => {
								smtp_has_full_inbox =
									val.as_bool().ok_or("has_full_inbox should be a boolean")?
							}
							"is_catch_all" => {
								smtp_is_catch_all =
									val.as_bool().ok_or("is_catch_all should be a boolean")?
							}
							"is_deliverable" => {
								smtp_is_deliverable =
									val.as_bool().ok_or("is_deliverable should be a boolean")?
							}
							"is_disabled" => {
								smtp_is_disabled =
									val.as_bool().ok_or("is_disabled should be a boolean")?
							}
							_ => {}
						}
					}
				}
				"syntax" => {
					let syntax_obj = val.as_object().ok_or("syntax field should be an object")?;
					for (key, val) in syntax_obj.keys().zip(syntax_obj.values()) {
						match key.as_str() {
							"error" => error = Some(val.to_string()),
							"is_valid_syntax" => {
								syntax_is_valid_syntax =
									val.as_bool().ok_or("is_valid_syntax should be a boolean")?
							}
							"username" => {
								syntax_username = val
									.as_str()
									.ok_or("username should be a string")?
									.to_string()
							}
							"domain" => {
								syntax_domain =
									val.as_str().ok_or("domain should be a string")?.to_string()
							}
							_ => {}
						}
					}
				}
				// ignore unknown fields
				_ => {}
			}
		}

		Ok(JobResultCsvResponse {
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
			syntax_domain,
			syntax_is_valid_syntax,
			syntax_username,
			error,
		})
	}
}
