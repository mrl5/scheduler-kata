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
    let _ = sqlx::query_file!("sql/enqueue.sql").execute(db).await?;

    Ok(())
}

pub async fn dequeue(db: &DB) -> anyhow::Result<Option<QueuePKey>> {
    let task_id: Option<QueuePKey> = sqlx::query_file_as!(QueuePKey, "sql/dequeue.sql")
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
    let retries: i16 = sqlx::query_file_scalar!(
        "sql/process_failure.init.sql",
        pkey.task_id,
        pkey.task_created_at
    )
    .fetch_one(&mut *tx)
    .await?;

    if retries >= RETRIES_HARD_LIMIT {
        finish_task(&mut *tx, pkey, TaskState::Failed).await?;
    } else {
        let _ = sqlx::query_file!(
            "sql/process_failure.retry.sql",
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

    let _ = sqlx::query_file!(
        "sql/finish_task.sql",
        pkey.task_id,
        pkey.task_created_at,
        state
    )
    .execute(db)
    .await?;

    Ok(())
}
