use chrono::{DateTime, Utc};
use common::db::DB;
use sqlx::PgExecutor;
use task_api::task_v1::model::TaskState;
use uuid::Uuid;

const RETRIES_HARD_LIMIT: i16 = 1;
const FINISH_STATES: [TaskState; 2] = [TaskState::Done, TaskState::Failed];

#[derive(sqlx::FromRow)]
pub struct QueuePKey {
    pub task_id: Uuid,
    pub task_created_at: DateTime<Utc>,
}

pub async fn enqueue(db: &DB) -> anyhow::Result<()> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO queue (task_id, task_created_at, not_before)
        SELECT id, created_at,
            CASE
                WHEN not_before IS NULL
                    THEN created_at
                ELSE not_before
            END
        FROM task_state WHERE state = $1::text
        ORDER BY id asc LIMIT 100
        ON CONFLICT DO NOTHING
    "#,
        TaskState::Created.to_string()
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn dequeue(db: &DB) -> anyhow::Result<Option<QueuePKey>> {
    let task_id: Option<QueuePKey> = sqlx::query_as!(
        QueuePKey,
        r#"
        WITH t AS (
            SELECT task_id as id, task_created_at AS created_at FROM queue
            WHERE is_running = false AND not_before <= now()
            LIMIT 1 FOR UPDATE SKIP LOCKED
        ) UPDATE queue SET is_running = true FROM t
        WHERE (task_id, task_created_at) = (t.id, t.created_at)
        RETURNING task_id, task_created_at
        "#
    )
    .fetch_optional(db)
    .await?;

    Ok(task_id)
}

pub async fn process_success(db: &DB, pkey: &QueuePKey) -> anyhow::Result<()> {
    finish_task(db, pkey, TaskState::Done).await?;

    Ok(())
}

pub async fn process_failure(err: anyhow::Error, db: &DB, pkey: &QueuePKey) -> anyhow::Result<()> {
    tracing::error!("task {} failed with: {}", &pkey.task_id, err);
    let mut tx = db.begin().await?;
    let retries: i16 = sqlx::query_scalar!(
        r#"
        UPDATE queue SET retries = retries + 1
        WHERE (task_id, task_created_at) = ($1::uuid, $2::timestamptz)
        RETURNING retries
        "#,
        pkey.task_id,
        pkey.task_created_at
    )
    .fetch_one(&mut *tx)
    .await?;

    if retries >= RETRIES_HARD_LIMIT {
        finish_task(&mut *tx, pkey, TaskState::Failed).await?;
    } else {
        let _ = sqlx::query!(
            r#"
            UPDATE queue SET is_running = false
            WHERE (task_id, task_created_at) = ($1::uuid, $2::timestamptz)
            "#,
            pkey.task_id,
            pkey.task_created_at
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn finish_task(
    db: impl PgExecutor<'_>,
    pkey: &QueuePKey,
    task_state: TaskState,
) -> anyhow::Result<()> {
    let mut state = task_state.to_string();
    if !FINISH_STATES.contains(&task_state) {
        tracing::error!("wrong task_state provided: {}", state);
        state = TaskState::Failed.to_string();
    }

    let _ = sqlx::query!(
        r#"
        WITH t AS (
            DELETE FROM queue
            WHERE (task_id, task_created_at) = ($1::uuid, $2::timestamptz)
            RETURNING task_id
        ) UPDATE task SET state = $3::text, inactive_since = now() FROM t
        WHERE id = t.task_id
        "#,
        pkey.task_id,
        pkey.task_created_at,
        state
    )
    .execute(db)
    .await?;

    Ok(())
}
