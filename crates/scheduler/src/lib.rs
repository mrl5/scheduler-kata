use common::db::DB;
use queue::enqueue;
use std::time::Duration;
use tokio::time::interval;

pub async fn run_scheduler(interval_ms: u64, db: DB) -> anyhow::Result<()> {
    tracing::debug!("interval: {} ms", interval_ms);
    let mut sleep = interval(Duration::from_millis(interval_ms));
    loop {
        sleep.tick().await;
        tracing::debug!("enqueuing");
        let _ = enqueue(&db)
            .await
            .map_err(|e| tracing::error!("enqueuing failed: {:?}", e));
    }
}
