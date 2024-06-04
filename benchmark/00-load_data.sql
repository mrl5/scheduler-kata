-- clean up
TRUNCATE TABLE data.task;

-- create tasks from past
ALTER TABLE data.task DISABLE TRIGGER t_update_uuid_before_insert;

WITH cte AS (
    SELECT
        (now() - (random() * '1 day'::interval)) AS created_at
    FROM
        generate_series(1, 1000000))
    INSERT INTO data.task (id, created_at, typ, state)
    SELECT
        public.ts_to_uuid_v7 (cte.created_at),
        cte.created_at,
        'type_c',
        'done'
    FROM
        cte;

ALTER TABLE data.task ENABLE TRIGGER t_update_uuid_before_insert;

-- stats
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

SELECT
    pg_stat_statements_reset ();

SELECT
    pg_stat_reset();

