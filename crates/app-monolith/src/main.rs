use axum::{routing::get, Extension};
use common::db;
use docs::{openapi, redoc};
use http_server::{router, server};
use std::net::SocketAddr;
use std::sync::Arc;
use task_api::router::task_v1_router;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const DEFAULT_PORT: &str = "8000";
const DOCS_PATH: &str = "/docs";
const OAS_PATH: &str = "/openapi.json";
const API_V1_PATH: &str = "/api/v1";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::debug!("tracing initiated");
    let port = std::env::var("API_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_owned());

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
    futures::try_join!(server_f)?;
    Ok(())
}
