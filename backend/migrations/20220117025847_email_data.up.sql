CREATE TABLE v0_bulk_jobs (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_records INTEGER NOT NULL
);
CREATE TABLE v0_bulk_results (
    id SERIAL PRIMARY KEY,
    job_id INTEGER,
    result JSONB,
    FOREIGN KEY (job_id) REFERENCES v0_bulk_jobs(id)
);
CREATE INDEX v0_job_emails ON v0_bulk_results USING HASH (job_id);