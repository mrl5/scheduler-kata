use std::env::var;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use worker::db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::debug!("tracing initiated");
    let db = db::connect(Some(env!("CARGO_PKG_NAME"))).await?;

    let workers_count: u8 = var("WORKERS_PER_INSTANCE")
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap_or(1);

    tracing::info!("workers enabled");
    worker::run_workers(db.clone(), workers_count).await?;

    Ok(())
}
