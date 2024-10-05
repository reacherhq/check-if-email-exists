-- Add migration script here
CREATE TABLE reacher_results (
    id SERIAL NOT NULL PRIMARY KEY,
    payload jsonb NOT NULL,
    worker TEXT NOT NULL,
    result jsonb,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
