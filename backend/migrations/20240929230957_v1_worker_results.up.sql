CREATE TABLE v1_bulk_job (
    id SERIAL PRIMARY KEY,
    total_records INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE v1_task_result (
    id SERIAL PRIMARY KEY,
    job_id INTEGER REFERENCES v1_bulk_job(id) ON DELETE CASCADE,
    payload JSONB NOT NULL,
    extra JSONB, -- any extra data that needs to be stored
    result JSONB,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_v1_task_result_job_id ON v1_task_result (job_id);
