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

use std::env;

use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tracing::{error, info};

mod sentry_util;
mod worker;

use sentry_util::setup_sentry;
use worker::process_check_email;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Setup sentry bug tracking.
	let _guard = setup_sentry();

	tracing_subscriber::fmt::init();

	// Make sure the worker is well configured.
	let addr = env::var("RCH_AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672".into());
	let backend_name = env::var("RCH_BACKEND_NAME").expect("RCH_BACKEND_NAME is not set");
	let verif_method: VerifMethod = env::var("RCH_VERIF_METHOD")
		.expect("RCH_VERIF_METHODS is not set")
		.as_str()
		.into();

	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		// At the moment the reactor is only available for unix.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(backend_name.clone().into());

	let conn = Connection::connect(&addr, options).await?;

	// Receive channel
	let channel = conn.create_channel().await?;
	info!(backend=?backend_name,state=?conn.status().state(), "Connected to AMQP broker");

	// Create queue "check_email.{Smtp,Headless}" with priority.
	let queue_name = format!("check_email.{:?}", verif_method);
	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), 5.into()); // https://www.rabbitmq.com/priority.html

	channel
		.queue_declare(
			&queue_name,
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			queue_args,
		)
		.await?;

	info!(queue=?queue_name, "Worker will start consuming messages");
	let mut consumer = channel
		.basic_consume(
			&queue_name,
			&backend_name,
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await?;

	while let Some(delivery) = consumer.next().await {
		if let Ok(delivery) = delivery {
			tokio::spawn(async move {
				let res = process_check_email(delivery).await;
				if let Err(err) = res {
					error!(error=?err, "Error processing message");
				}
			});
		}
	}

	Ok(())
}

// Verification method used by the worker.
#[derive(Debug, Clone, Copy)]
enum VerifMethod {
	// This worker will use a headless browser to verify emails.
	// Oftentimes, this also means that the worker doesn't have port 25 open.
	Headless,
	// This worker will use a SMTP server to verify emails.
	Smtp,
}

impl From<&str> for VerifMethod {
	fn from(s: &str) -> Self {
		match s {
			"Headless" => Self::Headless,
			"Smtp" => Self::Smtp,
			_ => panic!(
				"Unknown verification method {}, must be one of Headless, Smtp",
				s
			),
		}
	}
}
