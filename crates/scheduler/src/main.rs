use common::db;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const SCHEDULER_INTERVAL_MS: u64 = 2000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::debug!("tracing initiated");

    let db = db::connect(Some(env!("CARGO_PKG_NAME"))).await?;

    tracing::info!("scheduler enabled");
    scheduler::run_scheduler(SCHEDULER_INTERVAL_MS, db.clone()).await?;

    Ok(())
}
