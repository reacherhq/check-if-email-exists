// check-if-email-exists
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

//! Read provider- and domain-specific rules from a JSON, then match each
//! email verification to the domain/provider, and translate those rules into
//! code.
//!
//! IMPORTANT: This is still a beta feature, and probably needs refining.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Rules {
	/// Don't perform catch-all check.
	SkipCatchAll,
	/// Set smtp_timeout to 35s (if not overriden by user in request).
	SmtpTimeout35s,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RulesByDomain {
	pub rules: Vec<Rules>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllRules {
	/// Apply rules by domain name, i.e. after the @ symbol.
	pub by_domain: HashMap<String, RulesByDomain>,
	/// Apply rules by the MX host. Since each domain potentially has multiple
	/// MX records, we match by their suffix.
	pub by_mx_suffix: HashMap<String, RulesByDomain>,
}

pub(crate) static ALL_RULES: Lazy<AllRules> =
	Lazy::new(|| serde_json::from_str::<AllRules>(include_str!("rules.json")).unwrap());
