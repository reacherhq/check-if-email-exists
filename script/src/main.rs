
use sqlx::PgPool;
use sqlx::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().expect("Unable to load environment variables from .env file");

    let db_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL env var");
    let pool = PgPool::connect(&db_url).await?;

    // Define a struct to represent the rows returned by the query
    #[derive(sqlx::FromRow)]
    struct JobId {
        id: i32,
    }

    // Define the SQL query to identify job IDs where total_processed in email_results is equal to total_records in bulk_jobs
    let job_ids_to_delete_query = r#"
        SELECT b.id
        FROM bulk_jobs b
        JOIN (
            SELECT job_id, COUNT(*) as total_processed
            FROM email_results
            GROUP BY job_id
        ) e ON b.id = e.job_id
        WHERE b.total_records = e.total_processed
    "#;

    // Fetch the list of job IDs that match the criteria
    let job_ids_to_delete: Vec<JobId> = sqlx::query_as(job_ids_to_delete_query)
        .fetch_all(&pool)
        .await?;

    for job_id in job_ids_to_delete {
        // Before deleting from bulk_jobs, delete the corresponding records from email_results
        let delete_email_results_query = "DELETE FROM email_results WHERE job_id = $1";

        // Execute the delete query for email_results
        sqlx::query(delete_email_results_query)
            .bind(job_id.id)
            .execute(&pool)
            .await?;

        println!("Email results for job id {} deleted successfully.", job_id.id);

        // Now, you can safely delete the record from bulk_jobs
        let delete_bulk_jobs_query = "DELETE FROM bulk_jobs WHERE id = $1";

        // Execute the delete query for bulk_jobs
        sqlx::query(delete_bulk_jobs_query)
            .bind(job_id.id)
            .execute(&pool)
            .await?;

        println!("Bulk jobs record with id {} deleted successfully.", job_id.id);
    }

    Ok(())
}

