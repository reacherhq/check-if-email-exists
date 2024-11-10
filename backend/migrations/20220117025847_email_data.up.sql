CREATE TABLE bulk_jobs (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_records INTEGER NOT NULL
);
CREATE TABLE email_results (
    id SERIAL PRIMARY KEY,
    job_id INTEGER,
    result JSONB,
    FOREIGN KEY (job_id) REFERENCES bulk_jobs(id)
);
CREATE INDEX job_emails ON email_results USING HASH (job_id);