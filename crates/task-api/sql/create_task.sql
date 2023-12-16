with t as (
    insert into task (
        id,
        typ,
        not_before
    )
    values (
        $1,
        $2,
        $3
    ) returning created_at, not_before, id
) insert into task_bucket select created_at, not_before, id from t returning id
