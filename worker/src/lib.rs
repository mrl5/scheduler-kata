pub mod db;
pub mod error;

use db::DB;
use error::Error;
use rand::random;
use sqlx::postgres::PgListener;
use sqlx::types::Uuid;
use std::env::var;
use std::time::Duration;
use tokio::{
    spawn,
    time::{interval, sleep},
};
use tokio_util::sync::CancellationToken;

pub async fn run_workers(db: DB, workers_count: u8) -> anyhow::Result<()> {
    let mut workers = vec![];

    if workers_count > 0 {
        if workers_count > 1 {
            for _ in 1..workers_count {
                workers.push(run_worker(db.clone()).await?);
            }
        }
        run_worker_blocking(db.clone()).await?;
    }

    Ok(())
}

async fn run_worker(db: DB) -> anyhow::Result<()> {
    spawn(async move {
        let _ = run_worker_blocking(db).await;
    });

    Ok(())
}

async fn run_worker_blocking(db: DB) -> anyhow::Result<()> {
    let mut listener = PgListener::connect_with(&db).await?;
    let channel = "task.new";
    let poll_sleep: u64 = var("WORKER_POLL_SLEEP_MS")
        .unwrap_or("10".to_owned())
        .parse()
        .unwrap_or(10);
    let mut sleep = interval(Duration::from_millis(poll_sleep));
    loop {
        if let Some(pkey) = dequeue(&db, Duration::from_secs(10)).await? {
            let _ = work(&db, pkey)
                .await
                .map_err(|e| tracing::error!("{:?}", e));
        } else {
            listener.listen(channel).await?;
            let msg = listener.recv();
            let timeout = sleep.tick();

            tokio::select! {
                _ = msg => (),
                _ = timeout => (),
            }
            listener.unlisten(channel).await?;
        }
    }
}

async fn work(db: &DB, pkey: Uuid) -> anyhow::Result<()> {
    let mut listener = PgListener::connect_with(db).await?;
    let channel = "task.delete";

    let token = CancellationToken::new();
    let cloned_token = token.clone();

    tokio::spawn(async move {
        loop {
            listener.listen(channel).await?;
            let notification = listener.recv().await?;
            let task_id = notification.payload();
            if pkey.hyphenated().to_string() == task_id {
                token.cancel();
                return anyhow::Ok(());
            }
        }
    });

    match mock_job(pkey, cloned_token).await {
        Ok(_) => process_success(db, pkey).await,
        Err(e) => process_failure(e, db, pkey).await,
    }
}

async fn mock_job(pkey: Uuid, token: CancellationToken) -> anyhow::Result<()> {
    let rnd_factor: f32 = random();
    tokio::select! {
        _ = sleep(Duration::from_secs(5 + (4.0 * rnd_factor).floor() as u64)) => {
            match random() {
                true => {
                    tracing::info!("finished task {pkey}");
                    Ok(())
                },
                false => Err(Error::WorkerError("zonk".to_string()).into()),
            }
        },
        _ = token.cancelled() => {
            tracing::info!("cancelled task {pkey}");
            Ok(())
        },
    }
}

async fn dequeue(db: &DB, timeout: Duration) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query_file!("sql/dequeue.sql", timeout.as_secs() as i32)
        .fetch_optional(db)
        .await?;

    if let Some(task) = row {
        return Ok(task.id);
    }

    Ok(None)
}

async fn process_failure(err: anyhow::Error, db: &DB, task_id: Uuid) -> anyhow::Result<()> {
    tracing::error!("task {task_id} failed with: {err}");

    sqlx::query_file!("sql/finish.sql", task_id, "failed")
        .fetch_optional(db)
        .await?;

    Ok(())
}

async fn process_success(db: &DB, task_id: Uuid) -> anyhow::Result<()> {
    sqlx::query_file!("sql/finish.sql", task_id, "done")
        .fetch_optional(db)
        .await?;

    Ok(())
}
