-- https://www.cybertec-postgresql.com/en/view-permissions-and-row-level-security-in-postgresql/#view-permissions-in-security-invoker-views
-- https://www.postgresql.org/about/featurematrix/detail/389/
CREATE VIEW api.task WITH ( security_invoker = TRUE
) AS
SELECT
    id,
    typ,
    CASE WHEN read_ct = 0
        AND state IS NULL THEN
        'queued'
    WHEN vt >= clock_timestamp()
        AND read_ct > 0
        AND state IS NULL THEN
        'running'
    WHEN vt <= clock_timestamp()
    -- process each task only once
        AND read_ct > 0
        AND state IS NULL THEN
        'failed'
    ELSE
        state
    END,
    created_at,
    updated_at,
    CASE WHEN read_ct = 0 THEN
        read_ct
    ELSE
        read_ct - 1
    END AS retries
FROM
    data.task;

GRANT SELECT ON api.task TO data_ro_group;

CREATE VIEW api.create_task WITH ( security_invoker = TRUE
) AS
SELECT
    id,
    typ,
    vt AS not_before
FROM
    data.task;

GRANT SELECT ON api.create_task TO data_ro_group;

GRANT INSERT ON api.create_task TO data_rw_group;

