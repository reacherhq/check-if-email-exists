pub mod db;
pub mod error;
pub mod get;
pub mod post;
pub mod results;
pub mod task;

use std::env;
use check_if_email_exists::LOG_TARGET;
use sqlx::{Pool, Postgres};
use sqlxmq::{JobRegistry, JobRunnerHandle};
use tracing::info;

pub use task::email_verification_task;
use crate::http::v1::bulk::task::v1_email_verification_task;

pub async fn create_job_registry(pool: Pool<Postgres>) -> Result<JobRunnerHandle, sqlx::Error> {
    let min_task_conc = env::var("RCH_MINIMUM_CONCURRENT_TASK_FETCH")
        .ok()
        .and_then(|var| var.parse::<usize>().ok())
        .unwrap_or(10);
    let max_conc_task_fetch = env::var("RCH_MAXIMUM_CONCURRENT_TASK_FETCH")
        .ok()
        .and_then(|var| var.parse::<usize>().ok())
        .unwrap_or(20);

    // registry needs to be given list of jobs it can accept
    let registry = JobRegistry::new(&[email_verification_task, v1_email_verification_task]);

    // create runner for the message queue associated
    // with this job registry
    let runner = registry
        // Create a job runner using the connection pool.
        .runner(&pool)
        // Here is where you can configure the job runner
        // Aim to keep 10-20 jobs running at a time.
        .set_concurrency(min_task_conc, max_conc_task_fetch)
        // Start the job runner in the background.
        .run()
        .await?;

    info!(
        target: LOG_TARGET,
        "Bulk endpoints enabled with concurrency min={} to max={}.",
        min_task_conc, max_conc_task_fetch
    );

    Ok(runner)
}
