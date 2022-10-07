use crate::task::{Task, TaskType};
use crate::worker::some_computation;
use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use time::OffsetDateTime;
use tokio::spawn;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::mpsc::{self};
use tokio::task::spawn_blocking;

lazy_static! {
    static ref TASK_EVENT_CHANNEL: (Sender<Task>, Receiver<Task>) = broadcast::channel::<Task>(16);
}

pub fn routes() -> Router {
    Router::new().route("/create", post(create_task))
}

#[derive(Deserialize)]
struct NewTaskReq {
    task_type: TaskType,
    #[serde(with = "time::serde::iso8601")]
    start_after: OffsetDateTime,
}

#[derive(Serialize)]
struct NewTaskRes {
    task_id: String,
    msg: String,
}

async fn create_task(Json(body): Json<NewTaskReq>) -> (StatusCode, Json<NewTaskRes>) {
    let tx = &TASK_EVENT_CHANNEL.0.clone();
    let t = Task::new(body.task_type, body.start_after);
    let task_id = t.id.to_string();

    let accept_msg = "Your task is queued".to_owned();
    let fail_msg =
        "Due to server error, we couldn't accept this request. Please try again later".to_owned();

    match tx.send(t) {
        Ok(_) => (
            StatusCode::ACCEPTED,
            Json(NewTaskRes {
                task_id,
                msg: accept_msg,
            }),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(NewTaskRes {
                task_id,
                msg: fail_msg,
            }),
        ),
    }
}

pub async fn run_scheduler() -> anyhow::Result<()> {
    let tx = &TASK_EVENT_CHANNEL.0.clone();
    let mut task_event_rx = tx.subscribe();
    let (worker_tx, mut worker_rx) = mpsc::unbounded_channel::<Task>();
    let mut pending_queue = VecDeque::<Task>::new();
    let mut processed_buffer = Vec::<Task>::new(); // todo: remove

    spawn(async move {
        loop {
            match task_event_rx.recv().await {
                Ok(task) => pending_queue.push_back(task),
                Err(e) => tracing::error!("{}", e),
                // note: how to handle such case?
                // maybe resubscribe with retries and then panic?
            }

            // note: propose to discuss potential issue with this solution
            // update: "Tokio will spawn more blocking threads when they are requested through this
            // function until the upper limit configured on the Builder is reached. After reaching
            // the upper limit, the tasks are put in a queue."
            // -> seems like saturation should not wreck havoc
            while let Some(task) = pending_queue.pop_front() {
                if !task.can_be_started() {
                    pending_queue.push_back(task);
                } else {
                    let tx = worker_tx.clone();
                    // note: issues when `some_computation` is async
                    // self note: if it'd be async does it even make sense to run spawn_blocking?
                    // IMO no
                    spawn_blocking(move || some_computation(task, tx));
                }
            }
        }
    });

    while let Some(res) = worker_rx.recv().await {
        processed_buffer.push(res);
    }

    Ok(())
}
