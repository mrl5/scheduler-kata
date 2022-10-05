use axum::{response::Json, routing::get, Router, Server};
use serde::Serialize;

const ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(run_healthcheck));

    println!("Starting server ...");
    let server = Server::bind(&ADDR.parse().unwrap()).serve(app.into_make_service());
    println!("Server running at {}", ADDR);
    server.await.unwrap();
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
