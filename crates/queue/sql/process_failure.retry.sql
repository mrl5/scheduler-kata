update queue set is_running = false
where (task_id, task_created_at) = ($1::uuid, $2::timestamptz)
