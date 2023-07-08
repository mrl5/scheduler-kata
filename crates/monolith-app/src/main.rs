use crate::docs::serve_oas;
use aide::axum::routing::get_with;
use aide::axum::ApiRouter;
use aide::openapi::{self, OpenApi};
use aide::transform::TransformOperation;
use axum::{response::Json, routing::get, Extension, Router, Server};
use hyper::Response;
use schemars::JsonSchema;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::OnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub mod docs;
pub mod error;

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
        let mut api = init_openapi();
        let router = get_router()
            .route("/openapi.json", get(serve_oas))
            .finish_api(&mut api)
            .layer(Extension(Arc::new(api)));
        run_server(address, router).await?;
        Ok(()) as anyhow::Result<()>
    };
    futures::try_join!(server_f)?;
    Ok(())
}

fn init_openapi() -> OpenApi {
    aide::gen::on_error(|error| {
        tracing::error!("Aide generation error: {error}");
    });
    aide::gen::extract_schemas(true);
    aide::gen::infer_responses(true);
    aide::gen::inferred_empty_response_status(204);

    aide::gen::in_context(|ctx| ctx.schema = schemars::gen::SchemaSettings::openapi3().into());

    OpenApi {
        info: openapi::Info {
            title: "Scheduler API".to_owned(),
            version: "0.1.0".to_owned(),
            description: Some("Scheduler Kata".to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Health {
    status: HealthStatus,
}

#[derive(Eq, Debug, Hash, PartialEq, Serialize, JsonSchema)]
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

pub fn run_healthcheck_docs(op: TransformOperation) -> TransformOperation {
    op.description("Get health status")
}

pub fn get_router() -> ApiRouter {
    ApiRouter::new().api_route("/health", get_with(run_healthcheck, run_healthcheck_docs))
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
