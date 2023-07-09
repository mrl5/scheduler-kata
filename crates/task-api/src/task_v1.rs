use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use chrono::{DateTime, Utc};
use common::db::DB;
use docs::{error::AppError, extractor::Json};
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
pub async fn create_task(
    Extension(db): Extension<DB>,
    Json(body): Json<CreateTaskReq>,
) -> impl IntoApiResponse {
    let id: Uuid = Ulid::new().into();

    let task = sqlx::query_as!(
        model::TaskId,
        r#"
        INSERT INTO task (
            id,
            typ,
            not_before
        )
        VALUES (
            $1,
            $2,
            $3
        )
        RETURNING id
        "#,
        id,
        body.task_type.to_string(),
        body.not_before
    )
    .fetch_one(&db)
    .await;

    if let Ok(t) = task {
        return (StatusCode::ACCEPTED, Json(t)).into_response();
    }
    tracing::error!("{:?}", task);
    let message = "Internal Server Error";
    return (StatusCode::INTERNAL_SERVER_ERROR, AppError::new(message)).into_response();
}
