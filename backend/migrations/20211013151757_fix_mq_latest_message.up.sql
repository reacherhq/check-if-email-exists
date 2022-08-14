CREATE OR REPLACE FUNCTION mq_latest_message(from_channel_name TEXT, from_channel_args TEXT)
RETURNS UUID AS $$
    SELECT COALESCE(
        (
            SELECT id FROM mq_msgs
            WHERE channel_name = from_channel_name
            AND channel_args = from_channel_args
            AND after_message_id IS NOT NULL
            AND id != uuid_nil()
            AND NOT EXISTS(
                SELECT * FROM mq_msgs AS mq_msgs2
                WHERE mq_msgs2.after_message_id = mq_msgs.id
            )
            ORDER BY created_at DESC
            LIMIT 1
        ),
        uuid_nil()
    )
$$ LANGUAGE SQL STABLE;