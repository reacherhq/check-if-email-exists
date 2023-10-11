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
pub enum Rule {
	/// Don't perform catch-all check.
	SkipCatchAll,
	/// Set the SMTP timeout to 45s.
	SmtpTimeout45s,
}

#[derive(Debug, Deserialize, Serialize)]
struct RulesByDomain {
	rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AllRules {
	/// Apply rules by domain name, i.e. after the @ symbol.
	by_domain: HashMap<String, RulesByDomain>,
	/// Apply rules by the MX host. Since each domain potentially has multiple
	/// MX records, we match by their suffix.
	by_mx_suffix: HashMap<String, RulesByDomain>,
}

static ALL_RULES: Lazy<AllRules> =
	Lazy::new(|| serde_json::from_str::<AllRules>(include_str!("rules.json")).unwrap());

fn does_domain_have_rule(domain: &str, rule: &Rule) -> bool {
	if let Some(v) = ALL_RULES.by_domain.get(domain) {
		return v.rules.contains(rule);
	}

	false
}

fn does_mx_have_rule(host: &str, rule: &Rule) -> bool {
	for (k, v) in ALL_RULES.by_mx_suffix.iter() {
		if host.ends_with(k) {
			return v.rules.contains(rule);
		}
	}

	false
}

/// Check if either the domain or the MX host has any given rule.
pub fn has_rule(domain: &str, host: &str, rule: &Rule) -> bool {
	does_domain_have_rule(domain, rule) || does_mx_have_rule(host, rule)
}
