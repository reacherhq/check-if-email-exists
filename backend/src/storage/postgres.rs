use super::error::StorageError;
use crate::worker::do_work::{CheckEmailJobId, CheckEmailTask, TaskError};
use check_if_email_exists::{CheckEmailOutput, LOG_TARGET};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{debug, info};

#[derive(Debug)]
pub struct PostgresStorage {
	pub pg_pool: PgPool,
	extra: Option<serde_json::Value>,
}

impl PostgresStorage {
	pub async fn new(db_url: &str, extra: Option<serde_json::Value>) -> Result<Self, StorageError> {
		debug!(target: LOG_TARGET, "Connecting to DB: {}", db_url);
		// create connection pool with database
		// connection pool internally the shared db connection
		// with arc so it can safely be cloned and shared across threads
		let pg_pool = PgPoolOptions::new().connect(db_url).await?;

		sqlx::migrate!("./migrations").run(&pg_pool).await?;

		info!(target: LOG_TARGET, table="v1_task_result", "Connected to DB, Reacher will write verification results to DB");

		Ok(Self { pg_pool, extra })
	}

	pub async fn store(
		&self,
		task: &CheckEmailTask,
		worker_output: &Result<CheckEmailOutput, TaskError>,
		extra: Option<serde_json::Value>,
	) -> Result<(), StorageError> {
		let payload_json = serde_json::to_value(task)?;

		match worker_output {
			Ok(output) => {
				let output_json = serde_json::to_value(output)?;

				sqlx::query!(
					r#"
					INSERT INTO v1_task_result (payload, job_id, extra, result)
					VALUES ($1, $2, $3, $4)
					RETURNING id
					"#,
					payload_json,
					match task.job_id {
						CheckEmailJobId::Bulk(job_id) => Some(job_id),
						CheckEmailJobId::SingleShot => None,
					},
					extra,
					output_json,
				)
				.fetch_one(&self.pg_pool)
				.await?;
			}
			Err(err) => {
				sqlx::query!(
					r#"
					INSERT INTO v1_task_result (payload, job_id, extra, error)
					VALUES ($1, $2, $3, $4)
					RETURNING id
					"#,
					payload_json,
					match task.job_id {
						CheckEmailJobId::Bulk(job_id) => Some(job_id),
						CheckEmailJobId::SingleShot => None,
					},
					extra,
					err.to_string(),
				)
				.fetch_one(&self.pg_pool)
				.await?;
			}
		}

		debug!(target: LOG_TARGET, email=?task.input.to_email, "Wrote to DB");

		Ok(())
	}

	pub fn get_extra(&self) -> Option<serde_json::Value> {
		self.extra.clone()
	}
}
