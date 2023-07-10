use chrono::{DateTime, Utc};
use common::db::DB;
use uuid::Uuid;

use super::model;

pub async fn create_task(
    db: DB,
    id: Uuid,
    task_type: model::TaskType,
    not_before: Option<DateTime<Utc>>,
) -> anyhow::Result<model::TaskId> {
    Ok(sqlx::query_as!(
        model::TaskId,
        r#"
        INSERT INTO task (
            id,
            typ,
            not_before
        )
        VALUES (
            $1,
            $2,
            $3
        )
        RETURNING id
        "#,
        id,
        task_type.to_string(),
        not_before
    )
    .fetch_one(&db)
    .await?)
}

pub async fn show_task(db: DB, id: Uuid) -> anyhow::Result<Option<model::TaskView>> {
    Ok(sqlx::query_as!(
        model::TaskView,
        r#"
        SELECT
            id,
            typ,
            state,
            created_at,
            not_before,
            inactive_since
        FROM task_state
        WHERE id = $1::uuid
        "#,
        id,
    )
    .fetch_optional(&db)
    .await?)
}

pub async fn delete_task(
    db: DB,
    id: Uuid,
    forbidden_states: &[String],
) -> anyhow::Result<Option<model::TaskSnapshot>> {
    Ok(sqlx::query_as!(
        model::TaskSnapshot,
        r#"
        WITH deleted_task as (
            UPDATE task
            SET inactive_since = now(), state = $1
            FROM (
                SELECT id as task_id FROM task_state
                WHERE id = $2::uuid
                    AND state != ANY($3)
                    AND inactive_since IS NULL
            ) as t
            WHERE id = t.task_id
            RETURNING id, state, inactive_since
        ) SELECT id, state, inactive_since FROM (
        SELECT id, state, inactive_since FROM deleted_task
        UNION ALL
        SELECT id, state, inactive_since FROM task
        WHERE id = $2::uuid AND state = $1
        ) t
        "#,
        model::TaskState::Deleted.to_string(),
        id,
        forbidden_states,
    )
    .fetch_optional(&db)
    .await?)
}

pub async fn does_task_exist(db: DB, id: Uuid) -> anyhow::Result<bool> {
    let res = sqlx::query!(
        r#"
        SELECT 1 as t FROM task WHERE id = $1::uuid
        "#,
        id,
    )
    .fetch_optional(&db)
    .await?;

    Ok(res.is_some())
}
