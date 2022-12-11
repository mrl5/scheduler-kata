use axum::{http::StatusCode, Extension, Json};
use chrono::{DateTime, Utc};
use common::{
    db::DB,
    error::{Error, JsonResult},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

pub mod model;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListTasksResp {
    pub tasks: Vec<model::Task>,
}

pub async fn list_tasks(Extension(db): Extension<DB>) -> JsonResult<ListTasksResp> {
    // todo: pagination
    // todo: filtering by state
    // todo: filtering by type
    let rows = sqlx::query_as!(
        model::Task,
        r#"
        SELECT
            id,
            typ as "typ: model::TaskType",
            state as "state: model::TaskState",
            created_at,
            deleted_at,
            not_before
        FROM task
        ORDER BY id desc
        "#,
    )
    .fetch_all(&db)
    .await?;

    let resp = ListTasksResp { tasks: rows };
    Ok(Json(resp))
}

#[derive(Deserialize)]
pub struct CreateTaskReq {
    pub task_type: model::TaskType,
    pub not_before: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct CreateTaskResp {
    task_id: Uuid,
    task_state: model::TaskState,
}

pub async fn create_task(
    Extension(db): Extension<DB>,
    Json(body): Json<CreateTaskReq>,
) -> Result<(StatusCode, Json<CreateTaskResp>), Error> {
    let id: Uuid = Ulid::new().into();
    let task = sqlx::query_as!(
        model::CreatedTask,
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
        RETURNING
            id,
            state as "state: model::TaskState"
        "#,
        id,
        body.task_type as model::TaskType,
        body.not_before
    )
    .fetch_one(&db)
    .await?;

    let resp = CreateTaskResp {
        task_id: task.id,
        task_state: task.state,
    };
    Ok((StatusCode::ACCEPTED, Json(resp)))
}
