CREATE VIEW api.task_state WITH ( security_invoker = TRUE
) AS
SELECT
    id,
    state,
    updated_at,
    vt
FROM
    data.task;

GRANT SELECT, UPDATE ON api.task_state TO data_rw_group;

CREATE FUNCTION api.delete_task (id uuid)
    RETURNS api.task_state
    AS $$
    WITH cte AS (
        SELECT
            t.id,
            t.state,
            v.state AS v_state
        FROM
            api.task_state t
            JOIN api.task v ON t.id = v.id
        WHERE
            t.id = delete_task.id)
    UPDATE
        api.task_state t
    SET
        updated_at = clock_timestamp(),
        vt = clock_timestamp(),
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

-- missing: state IS NULL
CREATE OR REPLACE FUNCTION worker.dequeue (timeout int)
    RETURNS SETOF worker.task
    AS $$
    WITH cte AS (
        SELECT
            id
        FROM
            worker.task
        WHERE
            state IS NULL
            AND vt <= clock_timestamp()
            -- no retries
            AND read_ct < 1
        ORDER BY
            id ASC
        LIMIT 1
        FOR UPDATE
            SKIP LOCKED)
    UPDATE
        worker.task t
    SET
        vt = clock_timestamp() + timeout::text::interval,
        read_ct = read_ct + 1,
        updated_at = clock_timestamp()
    FROM
        cte
    WHERE
        t.id = cte.id
    RETURNING
        t.*
$$
LANGUAGE sql
VOLATILE;

