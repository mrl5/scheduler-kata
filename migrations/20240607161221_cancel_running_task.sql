ALTER TABLE data.task
    DROP CONSTRAINT state_check;

ALTER TABLE data.task
    ADD CONSTRAINT state_check CHECK (state IN ('cancelled', 'deleted', 'done', 'failed') OR state IS NULL);

CREATE OR REPLACE FUNCTION api.delete_task (id uuid)
    RETURNS api_util.task_state
    AS $$
    SELECT
        pg_notify('task.delete', id::text);
        WITH cte AS (
            SELECT
                t.id,
                t.state,
                v.state AS v_state
            FROM
                api_util.task_state t
                JOIN api.task v ON t.id = v.id
            WHERE
                t.id = delete_task.id)
        UPDATE
            api_util.task_state t
        SET
            updated_at = clock_timestamp(),
            state = CASE WHEN cte.v_state = 'running' THEN
                'cancelled'
            ELSE
                'deleted'
            END
        FROM
            cte
        WHERE
            t.id = delete_task.id
            AND cte.v_state = ANY ('{"queued", "running"}')
        RETURNING
            t.*
$$
LANGUAGE sql
VOLATILE;

