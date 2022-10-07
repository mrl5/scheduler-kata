use axum::{response::Json, routing::get, Router, Server};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::debug!("tracing initiated");
    let middleware_stack = ServiceBuilder::new().layer(TraceLayer::new_for_http().on_request(()));
    // note: issues with setting up the logs
    // update: I was probably confused with my own tests ... :/

    let app = Router::new()
        .route("/health", get(run_healthcheck))
        .layer(middleware_stack);

    tracing::info!("Starting server ...");
    let server = Server::bind(&ADDR.parse()?).serve(app.into_make_service());

    println!("Server running at {}", ADDR);
    futures::try_join!(server)?;
    Ok(())
}

#[derive(Serialize)]
struct Health {
    status: HealthStatus,
}

#[derive(Eq, Debug, Hash, PartialEq, Serialize)]
enum HealthStatus {
    Healthy,
}

async fn run_healthcheck() -> Json<Health> {
    let check = Health {
        status: HealthStatus::Healthy,
    };
    Json(check)
}
