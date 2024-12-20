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

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use reacher_backend::config::BackendConfig;
	use reacher_backend::http::{create_routes, CheckEmailRequest, REACHER_SECRET_HEADER};
	use warp::http::StatusCode;
	use warp::test::request;

	const FOO_BAR_RESPONSE: &str = r#"{"input":"foo@bar","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false,"is_b2c":false,"gravatar_url":null,"haveibeenpwned":null},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":null,"domain":"","is_valid_syntax":false,"username":"","normalized_email":null,"suggestion":null}"#;
	const FOO_BAR_BAZ_RESPONSE: &str = r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false,"is_b2c":false,"gravatar_url":null,"haveibeenpwned":null},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo","normalized_email":"foo@bar.baz","suggestion":null}"#;

	fn create_backend_config(header_secret: &str) -> Arc<BackendConfig> {
		let mut config = BackendConfig::empty();
		config.header_secret = Some(header_secret.to_string());
		Arc::new(config)
	}

	#[tokio::test]
	async fn test_input_foo_bar() {
		let resp = request()
			.path("/v0/check_email")
			.method("POST")
			.header(REACHER_SECRET_HEADER, "foobar")
			.json(&serde_json::from_str::<CheckEmailRequest>(r#"{"to_email": "foo@bar"}"#).unwrap())
			.reply(&create_routes(create_backend_config("foobar")))
			.await;

		assert_eq!(resp.status(), StatusCode::OK, "{:?}", resp.body());
		assert!(resp.body().starts_with(FOO_BAR_RESPONSE.as_bytes()));
	}

	#[tokio::test]
	async fn test_input_foo_bar_baz() {
		let resp = request()
			.path("/v0/check_email")
			.method("POST")
			.header(REACHER_SECRET_HEADER, "foobar")
			.json(
				&serde_json::from_str::<CheckEmailRequest>(r#"{"to_email": "foo@bar.baz"}"#)
					.unwrap(),
			)
			.reply(&create_routes(create_backend_config("foobar")))
			.await;

		assert_eq!(resp.status(), StatusCode::OK, "{:?}", resp.body());
		assert!(resp.body().starts_with(FOO_BAR_BAZ_RESPONSE.as_bytes()));
	}

	#[tokio::test]
	async fn test_reacher_secret_missing_header() {
		let resp = request()
			.path("/v0/check_email")
			.method("POST")
			.json(
				&serde_json::from_str::<CheckEmailRequest>(r#"{"to_email": "foo@bar.baz"}"#)
					.unwrap(),
			)
			.reply(&create_routes(create_backend_config("foobar")))
			.await;

		assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{:?}", resp.body());
		assert_eq!(resp.body(), r#"Missing request header "x-reacher-secret""#);
	}

	#[tokio::test]
	async fn test_reacher_secret_wrong_secret() {
		let resp = request()
			.path("/v0/check_email")
			.method("POST")
			.header(REACHER_SECRET_HEADER, "barbaz")
			.json(
				&serde_json::from_str::<CheckEmailRequest>(r#"{"to_email": "foo@bar.baz"}"#)
					.unwrap(),
			)
			.reply(&create_routes(create_backend_config("foobar")))
			.await;

		assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{:?}", resp.body());
		assert_eq!(resp.body(), r#"Invalid request header "x-reacher-secret""#);
	}

	#[tokio::test]
	async fn test_reacher_secret_correct_secret() {
		let resp = request()
			.path("/v0/check_email")
			.method("POST")
			.header(REACHER_SECRET_HEADER, "foobar")
			.json(&serde_json::from_str::<CheckEmailRequest>(r#"{"to_email": "foo@bar"}"#).unwrap())
			.reply(&create_routes(create_backend_config("foobar")))
			.await;

		assert_eq!(resp.status(), StatusCode::OK, "{:?}", resp.body());
		assert!(resp.body().starts_with(FOO_BAR_RESPONSE.as_bytes()));
	}

	#[tokio::test]
	async fn test_reacher_to_mail_empty() {
		let resp = request()
			.path("/v0/check_email")
			.method("POST")
			.header(REACHER_SECRET_HEADER, "foobar")
			.json(&serde_json::from_str::<CheckEmailRequest>(r#"{"to_email": ""}"#).unwrap())
			.reply(&create_routes(create_backend_config("foobar")))
			.await;

		assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{:?}", resp.body());
		assert_eq!(resp.body(), r#"{"error":"to_email field is required."}"#);
	}
}
