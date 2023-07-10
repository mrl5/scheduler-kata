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
) -> Result<impl IntoApiResponse, StatusCode> {
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
    Query(id): Query<Id>,
) -> Result<impl IntoApiResponse, StatusCode> {
    let task_id = id.id;
    let task = sqlx::query_as!(
        model::TaskView,
        r#"
        SELECT
            id,
            typ,
            state,
            created_at,
            not_before,
            inactive_since
        FROM task_state
        WHERE id = $1::uuid
        "#,
        task_id,
    )
    .fetch_optional(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(t) = task {
        Ok(Json(t).into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, Json(task_id)).into_response())
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
    Query(id): Query<Id>,
) -> Result<impl IntoApiResponse, StatusCode> {
    let forbidden_states = vec![model::TaskState::Processing.to_string()];
    let task_id = id.id;
    let task = sqlx::query_as!(
        model::TaskSnapshot,
        r#"
        WITH deleted_task as (
            UPDATE task
            SET inactive_since = now(), state = $1
            FROM (
                SELECT id as task_id FROM task_state
                WHERE id = $2::uuid
                    AND state != ANY($3)
                    AND inactive_since IS NULL
            ) as t
            WHERE id = t.task_id
            RETURNING id, state, inactive_since
        ) SELECT id, state, inactive_since FROM (
        SELECT id, state, inactive_since FROM deleted_task
        UNION ALL
        SELECT id, state, inactive_since FROM task
        WHERE id = $2::uuid AND state = $1
        ) t
        "#,
        model::TaskState::Deleted.to_string(),
        task_id,
        &forbidden_states,
    )
    .fetch_optional(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(t) = task {
        return Ok(Json(t).into_response());
    }
    let task = sqlx::query!(
        r#"
        SELECT 1 as t FROM task WHERE id = $1::uuid
        "#,
        task_id,
    )
    .fetch_optional(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match task {
        Some(_) => Ok((
            StatusCode::FORBIDDEN,
            Json(format!("{} can't be deleted anymore", task_id)),
        )
            .into_response()),
        None => Ok((StatusCode::NOT_FOUND, Json(task_id)).into_response()),
    }
}
