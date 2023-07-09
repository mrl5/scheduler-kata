use aide::transform::TransformOperation;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use docs::extractor::Json;
use schemars::JsonSchema;
use serde::Deserialize;
use ulid::Ulid;
use uuid::Uuid;

mod model;

pub fn create_task_docs(op: TransformOperation) -> TransformOperation {
    op.description("Create new task")
        .response::<202, Json<model::TaskId>>()
}
#[derive(Deserialize, JsonSchema)]
pub struct CreateTaskReq {
    pub task_type: model::TaskType,
    pub not_before: Option<DateTime<Utc>>,
}
pub async fn create_task(Json(body): Json<CreateTaskReq>) -> (StatusCode, Json<model::TaskId>) {
    tracing::debug!("{} {:?}", body.task_type, body.not_before);
    let id: Uuid = Ulid::new().into();
    let task = model::TaskId { id };
    (StatusCode::ACCEPTED, Json(task))
}
