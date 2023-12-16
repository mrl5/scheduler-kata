with t as (
    select task_id as id, task_created_at as created_at
    from queue
    where is_running = false and not_before <= now()
    limit 1 for update skip locked
) update queue set is_running = true from t
where (task_id, task_created_at) = (t.id, t.created_at)
returning task_id, task_created_at
