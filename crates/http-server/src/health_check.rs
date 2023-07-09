use aide::transform::TransformOperation;
use axum::response::Json;
use schemars::JsonSchema;
use serde::Serialize;

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
    op.description("Health status")
}
