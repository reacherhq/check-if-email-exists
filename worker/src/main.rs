use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tracing::info;

use reacher_worker::worker::CheckEmailWorker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Set default target for tracing to "reacher"

	tracing_subscriber::fmt::init();

	let addr = std::env::var("RCH_AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672".into());
	let options = ConnectionProperties::default()
		// Use tokio executor and reactor.
		// At the moment the reactor is only available for unix.
		.with_executor(tokio_executor_trait::Tokio::current())
		.with_reactor(tokio_reactor_trait::Tokio);

	let conn = Connection::connect(&addr, options).await?;

	//receive channel
	let channel = conn.create_channel().await?;
	info!(addr=?addr,state=?conn.status().state(), "Connected to AMQP broker.");

	// Create queue "tasks" if not exists
	let queue = channel
		.queue_declare(
			"check_email",
			QueueDeclareOptions {
				durable: true,
				..Default::default()
			},
			FieldTable::default(),
		)
		.await?;

	let backend_name = &std::env::var("RCH_BACKEND_NAME").unwrap_or_else(|_| "rch-worker".into());
	info!(backend=?backend_name,queue=?queue.name().as_str(), "Worker will start consuming messages.");
	let mut consumer = channel
		.basic_consume(
			queue.name().as_str(),
			backend_name,
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await?;

	// Define workers:
	let check_email_worker = CheckEmailWorker::new();

	while let Some(delivery) = consumer.next().await {
		if let Ok(delivery) = delivery {
			check_email_worker.process_check_email(delivery).await?;
		}
	}

	Ok(())
}
