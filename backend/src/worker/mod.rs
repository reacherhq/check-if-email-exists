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

use check_if_email_exists::LOG_TARGET;
use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use tracing::{error, info};

pub mod check_email;
#[cfg(feature = "postgres")]
mod db;

use check_email::process_check_email;
#[cfg(feature = "postgres")]
use db::create_db;

pub async fn create_channel(
	backend_name: &str,
) -> Result<Channel, Box<dyn std::error::Error + Send + Sync>> {
	// Make sure the worker is well configured.
	let addr = env::var("RCH_AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672".into());

	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		// At the moment the reactor is only available for unix (ref: https://github.com/amqp-rs/reactor-trait/issues/1)
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(backend_name.into());

	let conn = Connection::connect(&addr, options).await?;

	// Receive channel
	let channel = conn.create_channel().await?;
	let concurrency = env::var("RCH_WORKER_CONCURRENCY")
		.ok()
		.and_then(|s| s.parse::<u16>().ok())
		.unwrap_or(10);
	channel
		.basic_qos(concurrency, BasicQosOptions { global: false })
		.await?;
	info!(target: LOG_TARGET, backend=?backend_name,state=?conn.status().state(), concurrency=?concurrency, "Connected to AMQP broker");

	Ok(channel)
}

pub async fn run_worker(
	channel: Channel,
	backend_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let verif_method: VerifMethod = env::var("RCH_VERIF_METHOD")
		.map(|s| s.as_str().into())
		.unwrap_or(VerifMethod::Smtp);

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

	info!(target: LOG_TARGET, queue=?queue_name, "Worker will start consuming messages");
	let mut consumer = channel
		.basic_consume(
			&queue_name,
			backend_name,
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await?;

	#[cfg(feature = "postgres")]
	let conn_pool = create_db().await?;

	while let Some(delivery) = consumer.next().await {
		if let Ok(delivery) = delivery {
			let channel = channel.clone();
			#[cfg(feature = "postgres")]
			let conn_pool = conn_pool.clone();
			tokio::spawn(async move {
				#[cfg(feature = "postgres")]
				let res = process_check_email(&channel, delivery, conn_pool).await;
				#[cfg(not(feature = "postgres"))]
				let res = process_check_email(&channel, delivery).await;
				if let Err(err) = res {
					error!(target: LOG_TARGET, error=?err, "Error processing message");
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
