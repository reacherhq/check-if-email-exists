-- Add migration script here
CREATE TABLE email_results (
    id SERIAL NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    is_reachable VARCHAR(10) NOT NULL,
    full_result jsonb NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);