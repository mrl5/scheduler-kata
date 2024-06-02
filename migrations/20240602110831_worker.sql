CREATE VIEW worker.task WITH ( security_invoker = TRUE
) AS
SELECT
    id,
    typ,
    state,
    vt,
    updated_at,
    read_ct
FROM
    data.task;

GRANT SELECT, UPDATE ON worker.task TO worker_group;

CREATE FUNCTION worker.dequeue (timeout int)
    RETURNS SETOF worker.task
    AS $$
    WITH cte AS (
        SELECT
            id
        FROM
            worker.task
        WHERE
            vt <= clock_timestamp()
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

CREATE FUNCTION worker.complete_task (id uuid)
    RETURNS SETOF worker.task
    AS $$
    UPDATE
        worker.task t
    SET
        vt = clock_timestamp(),
        updated_at = clock_timestamp(),
        state = 'done'
    WHERE
        t.id = complete_task.id
    RETURNING
        *
$$
LANGUAGE sql
VOLATILE;

