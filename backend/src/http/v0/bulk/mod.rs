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

mod db;
mod error;
pub mod get;
pub mod post;
pub mod results;
mod task;

use std::env;

use check_if_email_exists::LOG_TARGET;
use sqlx::{Pool, Postgres};
use sqlxmq::{JobRegistry, JobRunnerHandle};
use tracing::info;

pub use task::email_verification_task;

/// Create a job registry with one task: the email verification task.
pub async fn create_job_registry(pool: &Pool<Postgres>) -> Result<JobRunnerHandle, sqlx::Error> {
	let min_task_conc = env::var("RCH_MINIMUM_TASK_CONCURRENCY").map_or(10, |var| {
		var.parse::<usize>()
			.expect("Environment variable RCH_MINIMUM_TASK_CONCURRENCY should parse to usize")
	});
	let max_conc_task_fetch = env::var("RCH_MAXIMUM_CONCURRENT_TASK_FETCH").map_or(20, |var| {
		var.parse::<usize>()
			.expect("Environment variable RCH_MAXIMUM_CONCURRENT_TASK_FETCH should parse to usize")
	});

	// registry needs to be given list of jobs it can accept
	let registry = JobRegistry::new(&[email_verification_task]);

	// create runner for the message queue associated
	// with this job registry
	let registry = registry
		// Create a job runner using the connection pool.
		.runner(pool)
		// Here is where you can configure the job runner
		// Aim to keep 10-20 jobs running at a time.
		.set_concurrency(min_task_conc, max_conc_task_fetch)
		// Start the job runner in the background.
		.run()
		.await?;

	info!(
		target: LOG_TARGET,
		"Bulk endpoints enabled with concurrency min={min_task_conc} to max={max_conc_task_fetch}."
	);

	Ok(registry)
}
