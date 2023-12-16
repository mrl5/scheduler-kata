with q as (
    delete from task_bucket using (
        select id from task_bucket limit 100 for update skip locked
    ) t
    where t.id = task_bucket.id
    returning task_bucket.id, task_bucket.created_at, task_bucket.not_before
) insert into queue (task_id, task_created_at, not_before)
select id, created_at,
    case
        when not_before is null
            then created_at
        else not_before
    end
from q on conflict do nothing
