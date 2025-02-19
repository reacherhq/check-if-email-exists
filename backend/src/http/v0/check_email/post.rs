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

//! This file implements the `POST /v0/check_email` endpoint.

use check_if_email_exists::smtp::verif_method::VerifMethod;
use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailInputProxy, LOG_TARGET};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use warp::{http, Filter};

use crate::config::BackendConfig;
use crate::http::{check_header, ReacherResponseError};

/// The request body for the `POST /v0/check_email` endpoint.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CheckEmailRequest {
	pub to_email: String,
	pub from_email: Option<String>,
	pub hello_name: Option<String>,
	pub proxy: Option<CheckEmailInputProxy>,
	pub smtp_timeout: Option<Duration>,
	pub smtp_port: Option<u16>,
}

impl CheckEmailRequest {
	pub fn to_check_email_input(&self, config: Arc<BackendConfig>) -> CheckEmailInput {
		// The current behavior is a bit complex. If the proxy field is present,
		// we force use the proxy for all the verifications. If the proxy field is
		// not present, we use the default configuration for all the verifications.
		//
		// If the proxy field is unset, but one of the other fields (from_email,
		// hello_name, smtp_timeout, smtp_port) is set, we ignore those fields.
		let verif_method = if let Some(proxy) = &self.proxy {
			VerifMethod::new_with_same_config_for_all(
				Some(proxy.clone()),
				self.hello_name
					.clone()
					.unwrap_or_else(|| config.hello_name.clone()),
				self.from_email
					.clone()
					.unwrap_or_else(|| config.from_email.clone()),
				self.smtp_port.unwrap_or(25),
				self.smtp_timeout
					.or(config.smtp_timeout.map(Duration::from_secs)),
				1,
			)
		} else {
			config.get_verif_method()
		};

		CheckEmailInput {
			to_email: self.to_email.clone(),
			verif_method,
			sentry_dsn: config.sentry_dsn.clone(),
			backend_name: config.backend_name.clone(),
			webdriver_config: config.webdriver.clone(),
			..Default::default()
		}
	}
}

/// The main endpoint handler that implements the logic of this route.
async fn http_handler(
	config: Arc<BackendConfig>,
	body: CheckEmailRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
	// The to_email field must be present
	if body.to_email.is_empty() {
		Err(
			ReacherResponseError::new(http::StatusCode::BAD_REQUEST, "to_email field is required.")
				.into(),
		)
	} else {
		// Run the future to check an email.
		Ok(warp::reply::json(
			&check_email(&body.to_check_email_input(Arc::clone(&config))).await,
		))
	}
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email<'a>(
	config: Arc<BackendConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + 'a {
	warp::path!("v0" / "check_email")
		.and(warp::post())
		.and(check_header(Arc::clone(&config)))
		.and(with_config(config))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(http_handler)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log(LOG_TARGET))
}

/// Warp filter that adds the BackendConfig to the handler.
pub fn with_config(
	config: Arc<BackendConfig>,
) -> impl Filter<Extract = (Arc<BackendConfig>,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || Arc::clone(&config))
}
