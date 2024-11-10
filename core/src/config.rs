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

use crate::SentryConfig;

/// Configuration needed to run Reacher.
#[derive(Debug)]
pub struct ReacherConfig {
	/// Identifier for the service currently running Reacher.
	pub backend_name: String,
	/// The address of the WebDriver server.
	pub webdriver_addr: String,
	#[cfg(feature = "sentry")]
	pub sentry: Option<SentryConfig>,
}

impl Default for ReacherConfig {
	fn default() -> Self {
		ReacherConfig {
			backend_name: "backend-dev".into(),
			webdriver_addr: "http://localhost:9515".into(),
			#[cfg(feature = "sentry")]
			sentry: Default::default(),
		}
	}
}
