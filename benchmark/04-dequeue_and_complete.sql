SELECT
    set_config('my.vars.id', id::text, FALSE)
FROM (
    SELECT
        id
    FROM
        worker.dequeue (59));

SELECT
    worker.complete_task (current_setting('my.vars.id')::uuid);

