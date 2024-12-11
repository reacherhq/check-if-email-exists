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

use crate::config::{BackendConfig, CommercialLicenseTrialConfig};
use crate::worker::do_work::TaskError;
use check_if_email_exists::{CheckEmailOutput, LOG_TARGET};
use std::sync::Arc;
use tracing::debug;

/// If we're in the Commercial License Trial, we also store the
/// result by sending it to back to Reacher.
pub async fn send_to_reacher(
	config: Arc<BackendConfig>,
	email: &str,
	worker_output: &Result<CheckEmailOutput, TaskError>,
) -> Result<(), reqwest::Error> {
	if let Some(CommercialLicenseTrialConfig { api_token, url }) = &config.commercial_license_trial
	{
		let res = reqwest::Client::new()
			.post(url)
			.header("Authorization", api_token)
			.json(worker_output)
			.send()
			.await?;
		let res = res.text().await?;
		debug!(target: LOG_TARGET, email=email, res=res, "Sent result to Reacher Commercial License Trial");
	}

	Ok(())
}
