-- task state view

CREATE VIEW task_state AS
    SELECT id, typ, state, created_at, not_before, inactive_since
    FROM task WHERE state IS NOT NULL

    UNION ALL

    SELECT
        t.id,
        t.typ,

        -- state
        CASE
            WHEN t.id = q.task_id AND q.is_running IS true
                THEN 'running'
            WHEN t.id = q.task_id AND t.not_before > now()
                THEN 'deferred'
            WHEN t.id = q.task_id
                THEN 'pending'
            ELSE 'created'
        END as state,

        t.created_at,
        t.not_before,
        t.inactive_since
    FROM task t
    LEFT JOIN queue q on t.id = q.task_id
    WHERE t.state is NULL;
