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
use std::fmt::{Display, Formatter};

use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tracing::{error, info};

mod sentry_util;
mod worker;

use sentry_util::setup_sentry;
use worker::process_check_email;

#[derive(Debug, Clone, Copy)]
enum VerifMethod {
	Api,
	Headless,
	Smtp,
}

impl Display for VerifMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Api => write!(f, "Api"),
			Self::Headless => write!(f, "Headless"),
			Self::Smtp => write!(f, "Smtp"),
		}
	}
}

impl From<&str> for VerifMethod {
	fn from(s: &str) -> Self {
		match s {
			"Api" => Self::Api,
			"Headless" => Self::Headless,
			"Smtp" => Self::Smtp,
			_ => panic!(format!(
				"Unknown verification method {s}, must be one of Api, Headless, Smtp"
			)),
		}
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Setup sentry bug tracking.
	let _guard = setup_sentry();

	tracing_subscriber::fmt::init();

	// Make sure the worker is well configured.
	let addr = env::var("RCH_AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672".into());
	let backend_name = env::var("RCH_BACKEND_NAME").expect("RCH_BACKEND_NAME is not set");
	let verif_methods = env::var("RCH_VERIF_METHODS").expect("RCH_VERIF_METHODS is not set");
	let verif_methods: Vec<VerifMethod> = verif_methods.split(',').map(|x| x.into()).collect();

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

	verif_methods.iter().for_each(|verif_method| {
		let backend_name_clone = backend_name.clone();
		let channel_clone = channel.clone();
		tokio::spawn(async move {
			if let Err(err) = consume_queue(verif_method, &backend_name_clone, &channel_clone).await
			{
				error!(error=?err, "Error consuming queue");
			}
		});
	});

	Ok(())
}

/// Consumes the queue for the given verification method.
async fn consume_queue(
	verif_method: &VerifMethod,
	backend_name: &str,
	channel: &lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Create queue "check_email" with priority.
	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), 5.into()); // https://www.rabbitmq.com/priority.html

	let queue = channel
		.queue_declare(
			format!("check_email.{}", verif_method.to_string()).as_str(),
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			queue_args,
		)
		.await?;

	info!(queue=?queue.name().as_str(), "Worker will start consuming messages");
	let mut consumer = channel
		.basic_consume(
			queue.name().as_str(),
			backend_name,
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
