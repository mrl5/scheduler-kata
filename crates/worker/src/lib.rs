use common::{db::DB, error::Error};
use queue::{dequeue, process_failure, process_success, QueuePKey};
use rand::{thread_rng, Rng};
use std::thread::sleep;
use std::time::Duration;
use tokio::spawn;
use tokio::time::interval;

// todo: parametrize number of workers
pub async fn run_workers(db: DB) -> anyhow::Result<()> {
    let mut handles = vec![];

    handles.push(run_worker(db.clone()).await?);
    handles.push(run_worker(db.clone()).await?);
    handles.push(run_worker(db.clone()).await?);

    Ok(())
}

pub async fn run_worker(db: DB) -> anyhow::Result<()> {
    spawn(async move {
        let mut sleep = interval(Duration::from_millis(10));
        loop {
            // note: this is interesting - w/o this interval one cpu thread is always 100%
            sleep.tick().await;
            let _ = work(&db).await.map_err(|e| tracing::error!("{:?}", e));
        }
    });

    Ok(())
}

async fn work(db: &DB) -> anyhow::Result<()> {
    if let Some(pkey) = dequeue(db).await? {
        match mock_job(&pkey) {
            Ok(_) => process_success(db, &pkey).await?,
            Err(e) => process_failure(e, db, &pkey).await?,
        };
    }

    Ok(())
}

fn mock_job(_pkey: &QueuePKey) -> anyhow::Result<()> {
    let mut rng = thread_rng();

    sleep(Duration::from_secs(rng.gen_range(2..7)));
    match rng.gen_bool(0.5) {
        true => Ok(()),
        false => Err(Error::WorkerError("zonk".to_string()).into()),
    }
}
