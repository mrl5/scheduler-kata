use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension};
use chrono::{DateTime, Utc};
use common::db::{Pagination, DB};
use docs::{error::AppError, extractor::Json};
use schemars::JsonSchema;
use serde::Deserialize;
use ulid::Ulid;
use uuid::Uuid;

use self::db_service::does_task_exist;

mod db_service;
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
) -> Result<impl IntoApiResponse, StatusCode> {
    let id: Uuid = Ulid::new().into();

    let task = db_service::create_task(db, id, body.task_type, body.not_before)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::ACCEPTED, Json(task)).into_response())
}

pub fn show_task_docs(op: TransformOperation) -> TransformOperation {
    op.description("Show task details")
        .response::<200, Json<model::TaskDetails>>()
        .response::<404, Json<AppError>>()
}
#[derive(Deserialize, JsonSchema)]
pub struct Id {
    id: Uuid,
}
pub async fn show_task(
    Extension(db): Extension<DB>,
    Query(q): Query<Id>,
) -> Result<impl IntoApiResponse, StatusCode> {
    let id = q.id;
    let task = db_service::show_task(db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(t) = task {
        Ok(Json(t).into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, Json(id)).into_response())
    }
}

pub fn delete_task_docs(op: TransformOperation) -> TransformOperation {
    op.description("Show task details")
        .response::<200, Json<model::Task>>()
        .response_with::<403, Json<AppError>, _>(|res| {
            res.description("Task is processed and can't be deleted anymore")
        })
        .response::<404, Json<AppError>>()
}
pub async fn delete_task(
    Extension(db): Extension<DB>,
    Query(q): Query<Id>,
) -> Result<impl IntoApiResponse, StatusCode> {
    let forbidden_states = vec![model::TaskState::Processing.to_string()];
    let id = q.id;
    let task = db_service::delete_task(db.clone(), id, &forbidden_states)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(t) = task {
        return Ok(Json(t).into_response());
    }
    let task_exists = does_task_exist(db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if task_exists {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(format!("{} can't be deleted anymore", id)),
        )
            .into_response());
    }
    Ok((StatusCode::NOT_FOUND, Json(id)).into_response())
}

pub fn list_tasks_docs(op: TransformOperation) -> TransformOperation {
    op.description("List tasks with optional filtering")
        .response::<200, Json<model::TasksList>>()
}
pub async fn list_tasks(
    Extension(db): Extension<DB>,
    Query(pagination): Query<Pagination>,
    Query(task_filter): Query<model::TaskFilter>,
) -> Result<impl IntoApiResponse, StatusCode> {
    let list = db_service::list_tasks(db, pagination, task_filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list).into_response())
}
