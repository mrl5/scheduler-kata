UPDATE
    worker.task t
SET
    updated_at = clock_timestamp(),
    state = $2::text
WHERE
    t.id = $1::uuid;

