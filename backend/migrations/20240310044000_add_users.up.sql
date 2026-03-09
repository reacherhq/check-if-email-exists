-- Add reacher_users table
CREATE TABLE IF NOT EXISTS reacher_users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert the requested user
-- Hashed password for 'm1m2m3#NMS'
INSERT INTO reacher_users (email, password_hash) 
VALUES ('shamilabdulla@nexkhad.com', '$2y$12$N9qo8uLOickgx2ZMRZoMyeIjZAgJR.XmPPH6j.5OInyD20x5uWv3.')
ON CONFLICT (email) DO NOTHING;
