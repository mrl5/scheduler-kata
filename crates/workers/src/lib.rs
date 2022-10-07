use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;
use tasks::{Task, TaskType};
use time::OffsetDateTime;
use tokio::spawn;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::task::spawn_blocking;

pub async fn run_workers() -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<Task>();
    let mut pending_queue = get_tasks();
    let mut processed_buffer = Vec::<Task>::new();

    spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10));

        loop {
            interval.tick().await;

            // todo: propose to discuss potential issue with this solution
            while let Some(task) = pending_queue.pop_front() {
                if !task.can_be_started() {
                    pending_queue.push_back(task);
                } else {
                    let ctx = tx.clone();
                    // note: issues when `some_computation` is async
                    // self note: if it'd be async does it even make sense to run spawn_blocking?
                    // IMO no
                    spawn_blocking(move || some_computation(task, ctx));
                }
            }
        }
    });

    while let Some(res) = rx.recv().await {
        processed_buffer.push(res);
    }

    Ok(())
}

fn some_computation(mut task: Task, tx: UnboundedSender<Task>) -> anyhow::Result<()> {
    let block_period = Duration::from_secs(5);
    let id = task.id.to_string();

    let res = task.set_as_running();
    match res {
        Ok(_) => tracing::info!("started task {}", id),
        Err(_) => tracing::error!("failed to run task {}", id),
    }

    sleep(block_period);
    match task.set_as_completed() {
        Ok(_) => tracing::info!("completed task {}", id),
        Err(_) => tracing::error!("unable to mark task {} as completed", id),
    };

    Ok(tx.send(task)?)
}

// todo: delete
fn get_tasks() -> VecDeque<Task> {
    let mut tasks = VecDeque::<Task>::new();
    let t1 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
    let t2 = Task::new(TaskType::TypeB, OffsetDateTime::now_utc());
    let t3 = Task::new(TaskType::TypeC, OffsetDateTime::now_utc());

    tasks.push_back(t1);
    tasks.push_back(t2);
    tasks.push_back(t3);

    tasks
}
