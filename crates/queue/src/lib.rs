use common::db::DB;

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
        FROM task_state WHERE state = 'created'
        ORDER BY id asc LIMIT 100
        ON CONFLICT DO NOTHING
    "#
    )
    .execute(db)
    .await?;

    Ok(())
}
