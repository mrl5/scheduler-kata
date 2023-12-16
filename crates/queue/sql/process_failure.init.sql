update queue set retries = retries + 1
where (task_id, task_created_at) = ($1::uuid, $2::timestamptz)
returning retries
