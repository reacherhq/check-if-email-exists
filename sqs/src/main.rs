use std::sync::Arc;

use aws_sdk_sqs::types::Message;
use aws_sdk_sqs::Client;
use check_if_email_exists::check_email;
use lambda_runtime::Error;
use reacher_backend::config::load_config;
use reacher_backend::http::CheckEmailRequest;
use serde::Deserialize;
use tracing::info;

const QUEUE_URL: &str = "https://sqs.eu-west-3.amazonaws.com/430118836964/check-email-queue";

#[derive(Debug, Deserialize)]
struct CheckEmailTask {
	input: CheckEmailRequest,
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
	let (event, _context) = event.into_parts();
	Ok(json!({ "message": "Hello, world!", "input": event }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	tracing_subscriber::fmt::init();
	dotenv::dotenv().ok();

	run(service_fn(function_handler)).await

	let shared_config = aws_config::load_from_env().await;
	let client = Client::new(&shared_config);
	let backend_config = load_config().await?;

	if let Some(message) = fetch_message(&client).await? {
		if let Some(body) = &message.body {
			let task: CheckEmailTask = serde_json::from_str(body)?;
			info!(email = ?task.input.to_email, "Processing task");
			let output =
				check_email(&task.input.to_check_email_input(Arc::new(backend_config))).await;
			info!(email = ?output.input, is_reachable = ?output.is_reachable, "Task completed");

			// Delete the message after successful processing
			delete_message(&client, &message).await?;
		}
	} else {
		info!("No messages to process");
	}

	Ok(())
}

async fn fetch_message(client: &Client) -> Result<Option<Message>, aws_sdk_sqs::Error> {
	let response = client
		.receive_message()
		.queue_url(QUEUE_URL)
		.max_number_of_messages(1) // Fetch a single message
		.visibility_timeout(120) // Visibility timeout in seconds
		.send()
		.await?;

	// Return the first message or None
	Ok(response.messages.and_then(|mut msgs| msgs.pop()))
}

async fn delete_message(client: &Client, message: &Message) -> Result<(), aws_sdk_sqs::Error> {
	if let Some(receipt_handle) = &message.receipt_handle {
		client
			.delete_message()
			.queue_url(QUEUE_URL)
			.receipt_handle(receipt_handle)
			.send()
			.await?;
	}

	Ok(())
}
