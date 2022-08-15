CREATE OR REPLACE FUNCTION mq_clear(channel_names TEXT[])
RETURNS VOID AS $$
BEGIN
    WITH deleted_ids AS (
        DELETE FROM mq_msgs
        WHERE channel_name = ANY(channel_names)
          AND id != uuid_nil()
        RETURNING id
    )
    DELETE FROM mq_payloads WHERE id IN (SELECT id FROM deleted_ids);
END;
$$ LANGUAGE plpgsql;
COMMENT ON FUNCTION mq_clear IS
    'Deletes all messages with corresponding payloads from a list of channel names';


CREATE OR REPLACE FUNCTION mq_clear_all()
RETURNS VOID AS $$
BEGIN
    WITH deleted_ids AS (
        DELETE FROM mq_msgs
        WHERE id != uuid_nil()
        RETURNING id
    )
    DELETE FROM mq_payloads WHERE id IN (SELECT id FROM deleted_ids);
END;
$$ LANGUAGE plpgsql;
COMMENT ON FUNCTION mq_clear_all IS
    'Deletes all messages with corresponding payloads';
