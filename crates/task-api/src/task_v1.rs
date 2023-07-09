use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension};
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
        .response::<400, Json<AppError>>()
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
    (StatusCode::INTERNAL_SERVER_ERROR, AppError::new(message)).into_response()
}

pub fn show_task_docs(op: TransformOperation) -> TransformOperation {
    op.description("Show task details")
        .response::<200, Json<model::Task>>()
        .response::<404, Json<AppError>>()
}
#[derive(Deserialize, JsonSchema)]
pub struct Id {
    id: Uuid,
}
pub async fn show_task(Extension(db): Extension<DB>, Query(id): Query<Id>) -> impl IntoApiResponse {
    let task_id = id.id;
    let task = sqlx::query_as!(
        model::Task,
        r#"
        SELECT
            id,
            typ,
            state,
            created_at,
            not_before,
            inactive_since
        FROM task
        WHERE id = $1::uuid
        "#,
        task_id,
    )
    .fetch_optional(&db)
    .await;

    if let Ok(tsk) = task {
        if let Some(t) = tsk {
            return Json(t).into_response();
        } else {
            return (StatusCode::NOT_FOUND, Json(task_id)).into_response();
        }
    }
    tracing::error!("{:?}", task);
    let message = "Internal Server Error";
    (StatusCode::INTERNAL_SERVER_ERROR, AppError::new(message)).into_response()
}
