use check_if_email_exists::CheckEmailInput;
use lapin::{options::*, BasicProperties};
use reacher_worker::check_email::WorkerPayload;
use reacher_worker::config::load_config;
use reacher_worker::worker::setup_rabbit_mq;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let worker_config = load_config()?;
	let channel = setup_rabbit_mq(&worker_config).await?;

	let payloads: Vec<WorkerPayload> = env::args()
		.skip(1) // Path to the binary
		.map(|s| WorkerPayload {
			input: CheckEmailInput {
				to_email: s,
				..Default::default()
			},
			extra: None,
		})
		.collect();

	for payload in payloads {
		let payload_bz = serde_json::to_vec(&payload)?;
		channel
			.basic_publish(
				"",
				"check.*",
				BasicPublishOptions::default(),
				&payload_bz,
				BasicProperties::default(),
			)
			.await?;

		println!("Published message: {:?}", payload.input.to_email);
	}

	Ok(())
}
