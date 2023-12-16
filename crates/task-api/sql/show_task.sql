select
    id,
    typ,
    state,
    created_at,
    not_before,
    inactive_since
from task_state
where id = $1::uuid
