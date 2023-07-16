use axum::{routing::get, Extension};
use common::db;
use docs::{openapi, redoc};
use http_server::{router, server};
use scheduler::run_scheduler;
use std::env::var;
use std::net::SocketAddr;
use std::sync::Arc;
use task_api::router::task_v1_router;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use worker::run_workers;

const DEFAULT_PORT: &str = "8000";
const DOCS_PATH: &str = "/docs";
const OAS_PATH: &str = "/openapi.json";
const API_V1_PATH: &str = "/api/v1";
const SCHEDULER_INTERVAL_MS: u64 = 2000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::debug!("tracing initiated");
    let port = var("API_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_owned());

    let db = db::connect(Some(env!("CARGO_PKG_NAME"))).await?;

    let server_f = async {
        let address = SocketAddr::from(([0, 0, 0, 0], port.parse()?));
        let mut api = openapi::init_openapi();
        let router = router::get_router()
            .route(OAS_PATH, get(openapi::serve_oas))
            .nest(DOCS_PATH, redoc::get_router(OAS_PATH))
            .nest(format!("{API_V1_PATH}/task").as_str(), task_v1_router())
            .finish_api_with(&mut api, openapi::api_docs)
            .layer(Extension(Arc::new(api)));
        server::run_server(address, router, Some(DOCS_PATH), db.clone()).await?;
        Ok(()) as anyhow::Result<()>
    };

    let enable_scheduler: bool = var("ENABLE_SCHEDULER")
        .unwrap_or("false".to_owned())
        .parse()
        .unwrap_or(false);
    let scheduler_f = async {
        if enable_scheduler {
            tracing::info!("scheduler enabled");
            run_scheduler(SCHEDULER_INTERVAL_MS, db.clone()).await?;
        }
        Ok(()) as anyhow::Result<()>
    };

    let enable_workers: bool = var("ENABLE_WORKERS")
        .unwrap_or("false".to_owned())
        .parse()
        .unwrap_or(false);
    let workers_count: u8 = var("WORKERS_PER_INSTANCE")
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap_or(1);
    let workers_f = async {
        if enable_workers {
            tracing::info!("workers enabled");
            run_workers(db.clone(), workers_count).await?;
        }
        Ok(()) as anyhow::Result<()>
    };

    futures::try_join!(server_f, scheduler_f, workers_f)?;
    Ok(())
}
