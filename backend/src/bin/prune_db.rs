use log::info;
use sqlx::PgPool;
use sqlx::Result;

#[tokio::main]
async fn main() -> Result<()> {
	dotenv::dotenv().expect("Unable to load environment variables from .env file");
	env_logger::init(); // Initialize the logger

	let db_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL env var");
	let dry_mode: bool = std::env::var("DRY_RUN").is_ok();
	let days_old_str = std::env::var("DAYS_OLD").expect("Unable to read DAYS_OLD env var");
	let days_old: i32 = days_old_str
		.parse()
		.expect("Unable to parse DAYS_OLD as integer");

	let pool = PgPool::connect(&db_url).await?;

	// Fetch the list of job IDs that match the criteria
	let query = format!(
		"SELECT b.id
        FROM bulk_jobs b
        JOIN (
            SELECT job_id, COUNT(*) as total_processed
            FROM email_results
            GROUP BY job_id
        ) e ON b.id = e.job_id
        WHERE b.total_records = e.total_processed
        AND b.created_at <= current_date - interval '{} days'",
		days_old
	);

	let job_ids_to_delete: Vec<(i32,)> = sqlx::query_as(&query).fetch_all(&pool).await?;

	match (dry_mode, job_ids_to_delete.is_empty()) {
		(true, _) => info!("Job ids to delete {:?}", job_ids_to_delete),
		(false, true) => info!("No jobs to delete"),
		(false, false) => {
			// Start a transaction
			let tx = pool.begin().await?;

			// Before deleting from bulk_jobs, delete the corresponding records from email_results in a batch
			let delete_email_results_query =
				"DELETE FROM email_results WHERE job_id = ANY($1::int[])";

			// Convert job_ids_to_delete to Vec<i32> before binding
			let job_ids_to_delete_vec: Vec<i32> =
				job_ids_to_delete.iter().map(|&(id,)| id).collect();

			// Execute the delete query for email_results in a batch within the transaction
			sqlx::query(delete_email_results_query)
				.bind(&job_ids_to_delete_vec)
				.execute(&pool) // Use execute on the query builder
				.await?;

			info!(
				"Email results for job IDs {:?} deleted successfully.",
				job_ids_to_delete
			);

			// safely delete the records from bulk_jobs
			let delete_bulk_jobs_query = "DELETE FROM bulk_jobs WHERE id = ANY($1::int[])";

			// Execute the delete query for bulk_jobs in a batch within the transaction
			sqlx::query(delete_bulk_jobs_query)
				.bind(&job_ids_to_delete_vec)
				.execute(&pool) // Use execute on the query builder
				.await?;

			info!(
				"Bulk jobs records with IDs {:?} deleted successfully.",
				job_ids_to_delete
			);

			// Commit the transaction if both deletes are successful
			tx.commit().await?;
		}
	}

	Ok(())
}
