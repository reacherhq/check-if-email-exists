-- Add migration script here
CREATE TABLE v1_worker_results (
    id SERIAL NOT NULL PRIMARY KEY,
    payload jsonb NOT NULL,
    backend_name TEXT NOT NULL,
    result jsonb,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
