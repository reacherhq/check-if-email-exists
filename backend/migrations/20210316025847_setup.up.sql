CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- The UDT for creating messages
CREATE TYPE mq_new_t AS (
    -- Unique message ID
    id UUID,
    -- Delay before message is processed
    delay INTERVAL,
    -- Number of retries if initial processing fails
    retries INT,
    -- Initial backoff between retries
    retry_backoff INTERVAL,
    -- Name of channel
    channel_name TEXT,
    -- Arguments to channel
    channel_args TEXT,
    -- Interval for two-phase commit (or NULL to disable two-phase commit)
    commit_interval INTERVAL,
    -- Whether this message should be processed in order with respect to other
    -- ordered messages.
    ordered BOOLEAN,
    -- Name of message
    name TEXT,
    -- JSON payload
    payload_json TEXT,
    -- Binary payload
    payload_bytes BYTEA
);

-- Small, frequently updated table of messages
CREATE TABLE mq_msgs (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    attempt_at TIMESTAMPTZ DEFAULT NOW(),
    attempts INT NOT NULL DEFAULT 5,
    retry_backoff INTERVAL NOT NULL DEFAULT INTERVAL '1 second',
    channel_name TEXT NOT NULL,
    channel_args TEXT NOT NULL,
    commit_interval INTERVAL,
    after_message_id UUID DEFAULT uuid_nil() REFERENCES mq_msgs(id) ON DELETE SET DEFAULT
);

-- Insert dummy message so that the 'nil' UUID can be referenced
INSERT INTO mq_msgs (id, channel_name, channel_args, after_message_id) VALUES (uuid_nil(), '', '', NULL);

-- Internal helper function to check that a UUID is neither NULL nor NIL
CREATE FUNCTION mq_uuid_exists(
    id UUID
) RETURNS BOOLEAN AS $$
	SELECT id IS NOT NULL AND id != uuid_nil()
$$ LANGUAGE SQL IMMUTABLE;

-- Index for polling
CREATE INDEX ON mq_msgs(channel_name, channel_args, attempt_at) WHERE id != uuid_nil() AND NOT mq_uuid_exists(after_message_id);
-- Index for adding messages
CREATE INDEX ON mq_msgs(channel_name, channel_args, created_at, id) WHERE id != uuid_nil() AND after_message_id IS NOT NULL;

-- Index for ensuring strict message order
CREATE UNIQUE INDEX mq_msgs_channel_name_channel_args_after_message_id_idx ON mq_msgs(channel_name, channel_args, after_message_id);


-- Large, less frequently updated table of message payloads
CREATE TABLE mq_payloads(
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    payload_json JSONB,
    payload_bytes BYTEA
);

-- Internal helper function to return the most recently added message in a queue.
CREATE FUNCTION mq_latest_message(from_channel_name TEXT, from_channel_args TEXT)
RETURNS UUID AS $$
    SELECT COALESCE(
        (
            SELECT id FROM mq_msgs
            WHERE channel_name = from_channel_name
            AND channel_args = from_channel_args
            AND after_message_id IS NOT NULL
            AND id != uuid_nil()
            ORDER BY created_at DESC, id DESC
            LIMIT 1
        ),
        uuid_nil()
    )
$$ LANGUAGE SQL STABLE;

-- Internal helper function to randomly select a set of channels with "ready" messages.
CREATE FUNCTION mq_active_channels(channel_names TEXT[], batch_size INT)
RETURNS TABLE(name TEXT, args TEXT) AS $$
    SELECT channel_name, channel_args
    FROM mq_msgs
    WHERE id != uuid_nil()
    AND attempt_at <= NOW()
    AND (channel_names IS NULL OR channel_name = ANY(channel_names))
    AND NOT mq_uuid_exists(after_message_id)
    GROUP BY channel_name, channel_args
    ORDER BY RANDOM()
    LIMIT batch_size
$$ LANGUAGE SQL STABLE;

-- Main entry-point for job runner: pulls a batch of messages from the queue.
CREATE FUNCTION mq_poll(channel_names TEXT[], batch_size INT DEFAULT 1)
RETURNS TABLE(
    id UUID,
    is_committed BOOLEAN,
    name TEXT,
    payload_json TEXT,
    payload_bytes BYTEA,
    retry_backoff INTERVAL,
    wait_time INTERVAL
) AS $$
BEGIN
    RETURN QUERY UPDATE mq_msgs
    SET
        attempt_at = CASE WHEN mq_msgs.attempts = 1 THEN NULL ELSE NOW() + mq_msgs.retry_backoff END,
        attempts = mq_msgs.attempts - 1,
        retry_backoff = mq_msgs.retry_backoff * 2
    FROM (
        SELECT
            msgs.id
        FROM mq_active_channels(channel_names, batch_size) AS active_channels
        INNER JOIN LATERAL (
            SELECT * FROM mq_msgs
            WHERE mq_msgs.id != uuid_nil()
            AND mq_msgs.attempt_at <= NOW()
            AND mq_msgs.channel_name = active_channels.name
            AND mq_msgs.channel_args = active_channels.args
            AND NOT mq_uuid_exists(mq_msgs.after_message_id)
            ORDER BY mq_msgs.attempt_at ASC
            LIMIT batch_size
        ) AS msgs ON TRUE
        LIMIT batch_size
    ) AS messages_to_update
    LEFT JOIN mq_payloads ON mq_payloads.id = messages_to_update.id
    WHERE mq_msgs.id = messages_to_update.id
    RETURNING
        mq_msgs.id,
        mq_msgs.commit_interval IS NULL,
        mq_payloads.name,
        mq_payloads.payload_json::TEXT,
        mq_payloads.payload_bytes,
        mq_msgs.retry_backoff / 2,
        interval '0' AS wait_time;

    IF NOT FOUND THEN
        RETURN QUERY SELECT
            NULL::UUID,
            NULL::BOOLEAN,
            NULL::TEXT,
            NULL::TEXT,
            NULL::BYTEA,
            NULL::INTERVAL,
            MIN(mq_msgs.attempt_at) - NOW()
        FROM mq_msgs
        WHERE mq_msgs.id != uuid_nil()
        AND NOT mq_uuid_exists(mq_msgs.after_message_id)
        AND (channel_names IS NULL OR mq_msgs.channel_name = ANY(channel_names));
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Creates new messages
CREATE FUNCTION mq_insert(new_messages mq_new_t[])
RETURNS VOID AS $$
BEGIN
    PERFORM pg_notify(CONCAT('mq_', channel_name), '')
    FROM unnest(new_messages) AS new_msgs
    GROUP BY channel_name;

    IF FOUND THEN
        PERFORM pg_notify('mq', '');
    END IF;

    INSERT INTO mq_payloads (
        id,
        name,
        payload_json,
        payload_bytes
    ) SELECT
        id,
        name,
        payload_json::JSONB,
        payload_bytes
    FROM UNNEST(new_messages);

    INSERT INTO mq_msgs (
        id,
        attempt_at,
        attempts,
        retry_backoff,
        channel_name,
        channel_args,
        commit_interval,
        after_message_id
    )
    SELECT
        id,
        NOW() + delay + COALESCE(commit_interval, INTERVAL '0'),
        retries + 1,
        retry_backoff,
        channel_name,
        channel_args,
        commit_interval,
        CASE WHEN ordered
            THEN
                LAG(id, 1, mq_latest_message(channel_name, channel_args))
                OVER (PARTITION BY channel_name, channel_args, ordered ORDER BY id)
            ELSE
                NULL
            END
    FROM UNNEST(new_messages);
END;
$$ LANGUAGE plpgsql;

-- Commits messages previously created with a non-NULL commit interval.
CREATE FUNCTION mq_commit(msg_ids UUID[])
RETURNS VOID AS $$
BEGIN
    UPDATE mq_msgs
    SET
        attempt_at = attempt_at - commit_interval,
        commit_interval = NULL
    WHERE id = ANY(msg_ids)
    AND commit_interval IS NOT NULL;
END;
$$ LANGUAGE plpgsql;


-- Deletes messages from the queue. This occurs when a message has been
-- processed, or when it expires without being processed.
CREATE FUNCTION mq_delete(msg_ids UUID[])
RETURNS VOID AS $$
BEGIN
    PERFORM pg_notify(CONCAT('mq_', channel_name), '')
    FROM mq_msgs
    WHERE id = ANY(msg_ids)
    AND after_message_id = uuid_nil()
    GROUP BY channel_name;

    IF FOUND THEN
        PERFORM pg_notify('mq', '');
    END IF;

    DELETE FROM mq_msgs WHERE id = ANY(msg_ids);
    DELETE FROM mq_payloads WHERE id = ANY(msg_ids);
END;
$$ LANGUAGE plpgsql;


-- Can be called during the initial commit interval, or when processing
-- a message. Indicates that the caller is still active and will prevent either
-- the commit interval elapsing or the message being retried for the specified
-- interval.
CREATE FUNCTION mq_keep_alive(msg_ids UUID[], duration INTERVAL)
RETURNS VOID AS $$
    UPDATE mq_msgs
    SET
        attempt_at = NOW() + duration,
        commit_interval = commit_interval + ((NOW() + duration) - attempt_at)
    WHERE id = ANY(msg_ids)
    AND attempt_at < NOW() + duration;
$$ LANGUAGE SQL;


-- Called during lengthy processing of a message to checkpoint the progress.
-- As well as behaving like `mq_keep_alive`, the message payload can be
-- updated.
CREATE FUNCTION mq_checkpoint(
    msg_id UUID,
    duration INTERVAL,
    new_payload_json TEXT,
    new_payload_bytes BYTEA,
    extra_retries INT
)
RETURNS VOID AS $$
    UPDATE mq_msgs
    SET
        attempt_at = GREATEST(attempt_at, NOW() + duration),
        attempts = attempts + COALESCE(extra_retries, 0)
    WHERE id = msg_id;

    UPDATE mq_payloads
    SET
        payload_json = COALESCE(new_payload_json::JSONB, payload_json),
        payload_bytes = COALESCE(new_payload_bytes, payload_bytes)
    WHERE
        id = msg_id;
$$ LANGUAGE SQL;

