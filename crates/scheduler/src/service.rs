use crate::task::{flush, Task, TaskType, FS_PERSIST_PATH};
use crate::worker::some_computation;
use axum::http::StatusCode;
use axum::{routing::get, routing::post, Json, Router};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::spawn;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::mpsc::{self};
use tokio::task::spawn_blocking;
use uuid::Uuid;

lazy_static! {
    static ref CREATE_TASK_EVENT_CHANNEL: (Sender<Task>, Receiver<Task>) =
        broadcast::channel::<Task>(16);
}

pub fn routes() -> Router {
    Router::new()
        .route("/create", post(create_task))
        .route("/list", get(list_tasks))
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
    let tx = &CREATE_TASK_EVENT_CHANNEL.0.clone();
    let t = Task::new(body.task_type, body.start_after);
    let task_id = t.id.to_string();
    let task_type = t.task_type.to_owned();

    let accept_msg = "Your task is queued".to_owned();
    let queue_err_msg =
        "Due to server error, we couldn't accept this request. Please try again later".to_owned();
    let persist_err_msg = "Something went wrong on our end. If error persist please include task_id in your bug report".to_owned();

    tracing::debug!("persisting {:?}: {} ...", task_type, task_id);
    if let Err(e) = t.persist_blocking() {
        tracing::error!("{}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(NewTaskRes {
                task_id,
                msg: persist_err_msg,
            }),
        );
    }
    tracing::info!("persisted {:?}: {}", task_type, task_id);

    tracing::debug!("sending {:?}: {} ...", task_type, task_id);
    match tx.send(t) {
        Ok(_) => (
            StatusCode::ACCEPTED,
            Json(NewTaskRes {
                task_id,
                msg: accept_msg,
            }),
        ),
        Err(e) => {
            tracing::error!("{}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(NewTaskRes {
                    task_id,
                    msg: queue_err_msg,
                }),
            )
        }
    }
}

pub async fn run_scheduler() -> anyhow::Result<()> {
    let mut create_task_event_rx = CREATE_TASK_EVENT_CHANNEL.0.clone().subscribe();

    let (worker_receive_tx, mut worker_receive_rx) = mpsc::unbounded_channel::<Uuid>();
    let (worker_release_tx, mut worker_release_rx) = mpsc::unbounded_channel::<Task>();

    let mut pending_queue = VecDeque::<Task>::new();
    let mut received_journal = Vec::<Uuid>::new();
    let mut released_journal = Vec::<Uuid>::new();

    flush()?;

    spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        loop {
            // note: this is interesting - w/o this interval one cpu thread is always 100%
            interval.tick().await;

            if !create_task_event_rx.is_empty() {
                match create_task_event_rx.recv().await {
                    Ok(task) => {
                        let task_type = task.task_type.to_owned();
                        let task_id = task.id.to_string();
                        tracing::info!("received {:?}: {}", task_type, task_id);
                        pending_queue.push_back(task)
                    }
                    Err(e) => tracing::error!("{}", e),
                    // note: how to handle such case?
                    // maybe resubscribe with retries and then panic?
                }
            }

            let (mut ready_queue, pending): (VecDeque<Task>, VecDeque<Task>) =
                pending_queue.iter().partition(|task| task.can_be_started());
            pending_queue = pending;

            // note: propose to discuss potential issue with this solution
            // update: "Tokio will spawn more blocking threads when they are requested through this
            // function until the upper limit configured on the Builder is reached. After reaching
            // the upper limit, the tasks are put in a queue."
            // -> seems like saturation should not wreck havoc
            while let Some(task) = ready_queue.pop_front() {
                let rec_tx = worker_receive_tx.clone();
                let rel_tx = worker_release_tx.clone();
                // note: issues when `some_computation` is async
                // self note: if it'd be async does it even make sense to run spawn_blocking?
                // IMO no
                spawn_blocking(move || some_computation(task, rec_tx, rel_tx));
            }
        }
    });

    let task_receive_listener = async {
        while let Some(task) = worker_receive_rx.recv().await {
            received_journal.push(task);
        }

        anyhow::Ok(())
    };

    let task_release_listener = async {
        while let Some(task) = worker_release_rx.recv().await {
            released_journal.push(task.id);
        }

        anyhow::Ok(())
    };

    futures::try_join!(task_receive_listener, task_release_listener)?;

    Ok(())
}

#[derive(Serialize)]
struct ListTasksRes {
    tasks: Option<Vec<String>>,
}

// note: endpoint handler returning Result - how to do it?
async fn list_tasks() -> (StatusCode, Json<ListTasksRes>) {
    let path: PathBuf = FS_PERSIST_PATH.iter().collect();
    match read_dir(path) {
        Ok(iterator) => {
            let ids = iterator
                .map(|x| match x {
                    Ok(entry) => {
                        let p = entry.path();
                        let filename: &Path = p.as_ref();
                        let filename = filename
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("");
                        let id = filename.split('.').next().unwrap_or("");
                        id.to_owned()
                    }
                    Err(e) => {
                        tracing::error!("{}", e);
                        "".to_owned()
                    }
                })
                .filter(|x| !x.is_empty())
                .collect();
            (StatusCode::OK, Json(ListTasksRes { tasks: Some(ids) }))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ListTasksRes { tasks: None }),
            )
        }
    }
}
