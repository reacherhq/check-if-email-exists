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

use super::check_email::CheckEmailTask;
use super::check_email::TaskWebhook;
use crate::config::{BackendConfig, Queue};
use crate::http::CheckEmailRequest;
use anyhow::anyhow;
use check_if_email_exists::mx::check_mx;
use check_if_email_exists::syntax::check_syntax;
use check_if_email_exists::{is_gmail, is_hotmail_b2b, is_hotmail_b2c, is_yahoo, LOG_TARGET};
use lapin::message::Delivery;
use lapin::{options::*, Channel};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Deserialize, Serialize)]
pub struct PreprocessTask {
	pub input: CheckEmailRequest,
	// If the task is a part of a job, then this field will be set.
	pub job_id: Option<i32>,
	pub webhook: Option<TaskWebhook>,
}

/// Preprocess the email and send it to the appropriate queue for verification.
pub async fn do_preprocess_work(
	payload: &PreprocessTask,
	delivery: Delivery,
	channel: Arc<Channel>,
	config: Arc<BackendConfig>,
) -> Result<(), anyhow::Error> {
	let syntax = check_syntax(&payload.input.to_email);
	let mx = check_mx(&syntax).await?;
	// Get first hostname from MX records.
	let mx_hostname = mx
		.lookup?
		.iter()
		.next()
		.ok_or_else(|| anyhow!("No MX records found"))?
		.exchange()
		.to_string();

	let queue = match mx_hostname.as_str() {
		hostname if is_gmail(hostname) => Queue::GmailSmtp,
		hostname if is_hotmail_b2b(hostname) => Queue::HotmailB2BSmtp,
		hostname if is_hotmail_b2c(hostname) => Queue::HotmailB2CHeadless,
		hostname if is_yahoo(hostname) => Queue::YahooHeadless,
		_ => Queue::EverythingElseSmtp,
	};
	let check_email_input = payload.input.to_check_email_input(config);
	let check_email_task = CheckEmailTask {
		input: check_email_input,
		job_id: payload.job_id,
		webhook: payload.webhook.clone(),
	};
	let check_email_payload = serde_json::to_vec(&check_email_task)?;

	channel
		.basic_publish(
			"",
			format!("{}", queue).as_str(),
			BasicPublishOptions::default(),
			&check_email_payload,
			delivery.properties.clone(),
		)
		.await?
		.await?;

	delivery.ack(BasicAckOptions::default()).await?;
	debug!(target: LOG_TARGET, email=?payload.input.to_email, queue=?queue.to_string(), "Message do_preprocess_worked");

	Ok(())
}
