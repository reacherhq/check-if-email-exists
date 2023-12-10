use std::env;

use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tracing::{error, info};

mod worker;

use worker::process_check_email;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	tracing_subscriber::fmt::init();

	let addr = env::var("RCH_AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672".into());
	let backend_name = env::var("RCH_BACKEND_NAME").expect("RCH_BACKEND_NAME is not set");
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		// At the moment the reactor is only available for unix.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio)
		.with_connection_name(backend_name.clone().into());

	let conn = Connection::connect(&addr, options).await?;

	// Receive channel
	let channel = conn.create_channel().await?;
	info!(state=?conn.status().state(), "Connected to AMQP broker");

	// Create queue "check_email" with priority.
	let mut queue_args = FieldTable::default();
	queue_args.insert("x-max-priority".into(), 5.into()); // https://www.rabbitmq.com/priority.html

	let queue = channel
		.queue_declare(
			"check_email",
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			queue_args,
		)
		.await?;

	info!(backend=?backend_name,queue=?queue.name().as_str(), "Worker will start consuming messages");
	let mut consumer = channel
		.basic_consume(
			queue.name().as_str(),
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
