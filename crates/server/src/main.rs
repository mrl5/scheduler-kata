use ::tracing::{field, Span};
use axum::{response::Json, routing::get, Router, Server};
use hyper::Response;
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::trace::{MakeSpan, OnResponse, TraceLayer};

const ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    // todo: LOGS? where are logs? :D
    tracing_subscriber::fmt::init();
    let middleware_stack = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .on_response(MyOnResponse {})
            .make_span_with(MyMakeSpan {})
            .on_request(()),
    );
    // note: issues with setting up the logs

    let app = Router::new()
        .route("/health", get(run_healthcheck))
        .layer(middleware_stack);

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

#[derive(Clone)]
pub struct MyMakeSpan {}

impl<B> MakeSpan<B> for MyMakeSpan {
    fn make_span(&mut self, request: &hyper::Request<B>) -> Span {
        tracing::info_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            username = field::Empty,
            workspace_id = field::Empty,
            email = field::Empty,
        )
    }
}
