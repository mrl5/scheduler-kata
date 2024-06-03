CREATE VIEW api_util.task_state WITH ( security_invoker = TRUE
) AS
SELECT
    id,
    state,
    updated_at,
    vt
FROM
    data.task;

GRANT SELECT, UPDATE ON api_util.task_state TO data_rw_group;

CREATE FUNCTION api.delete_task (id uuid)
    RETURNS api_util.task_state
    AS $$
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
        state = 'deleted'
    FROM
        cte
    WHERE
        t.id = delete_task.id
        AND cte.v_state = 'queued'
    RETURNING
        t.*
$$
LANGUAGE sql
VOLATILE;

