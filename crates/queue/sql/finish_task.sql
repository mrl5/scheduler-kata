with t as (
    delete from queue
    where (task_id, task_created_at) = ($1::uuid, $2::timestamptz)
    returning task_id
) update task set state = $3::text, inactive_since = now() from t
where id = t.task_id
