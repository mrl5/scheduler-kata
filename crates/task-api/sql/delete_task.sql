with deleted_task as (
    update task
    set inactive_since = now(), state = $1
    from (
        select id as task_id from task_state
        where id = $2::uuid
            and state != any($3)
            and inactive_since is null
    ) as t
    where id = t.task_id
    returning id, state, inactive_since
), dt as (
    delete from task_bucket using deleted_task
    where task_bucket.id = deleted_task.id
) select id, state, inactive_since from (
    select id, state, inactive_since from deleted_task
    union all
    select id, state, inactive_since from task
    where id = $2::uuid and state = $1
) t
