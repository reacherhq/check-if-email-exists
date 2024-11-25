-- Add down migration script here
DROP INDEX IF EXISTS v1_worker_results_job_id;
DROP TABLE IF EXISTS v1_task_result;
DROP TABLE IF EXISTS v1_bulk_job;
