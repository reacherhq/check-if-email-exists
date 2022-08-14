-- Deletes all messages from a list of channel names.
CREATE FUNCTION mq_clear(channel_names TEXT[])
RETURNS VOID AS $$
BEGIN
    WITH deleted_ids AS (
        DELETE FROM mq_msgs WHERE channel_name = ANY(channel_names) RETURNING id
    )
    DELETE FROM mq_payloads WHERE id IN (SELECT id FROM deleted_ids);
END;
$$ LANGUAGE plpgsql;

-- Deletes all messages.
CREATE FUNCTION mq_clear_all()
RETURNS VOID AS $$
BEGIN
    WITH deleted_ids AS (
        DELETE FROM mq_msgs RETURNING id
    )
    DELETE FROM mq_payloads WHERE id IN (SELECT id FROM deleted_ids);
END;
$$ LANGUAGE plpgsql;
