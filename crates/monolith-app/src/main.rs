use axum::{response::Json, routing::get, Router, Server};
use hyper::Response;
use serde::Serialize;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::OnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const DEFAULT_PORT: &str = "8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::debug!("tracing initiated");
    let port = std::env::var("API_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_owned());
    let server_f = async {
        let address = SocketAddr::from(([0, 0, 0, 0], port.parse()?));
        let router = get_router();
        run_server(address, router).await?;
        Ok(()) as anyhow::Result<()>
    };
    futures::try_join!(server_f)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Health {
    status: HealthStatus,
}

#[derive(Eq, Debug, Hash, PartialEq, Serialize)]
pub enum HealthStatus {
    Healthy,
}

pub async fn run_healthcheck() -> Json<Health> {
    let check = Health {
        status: HealthStatus::Healthy,
    };
    tracing::info!("{:?}", check);
    Json(check)
}

pub fn get_router() -> Router {
    Router::new().route("/health", get(run_healthcheck))
}

pub async fn run_server(address: SocketAddr, router: Router) -> anyhow::Result<()> {
    let middleware_stack = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .on_request(())
            .on_response(MyOnResponse {}),
    );

    let app = router.layer(middleware_stack);

    tracing::info!("Starting server ...");
    let server = async {
        Server::bind(&address)
            .serve(app.into_make_service())
            .await?;
        Ok(()) as anyhow::Result<()>
    };

    println!("Server running at http://{}", address);
    server.await?;
    Ok(())
}

#[derive(Clone)]
pub struct MyOnResponse {}

impl<B> OnResponse<B> for MyOnResponse {
    fn on_response(
        self,
        response: &Response<B>,
        latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        tracing::info!(
            latency = latency.as_millis(),
            status = response.status().as_u16(),
            "response"
        )
    }
}
