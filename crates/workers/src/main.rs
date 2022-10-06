use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;
use tasks::{Task, TaskType};
use time::OffsetDateTime;
use tokio::spawn;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::task::spawn_blocking;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::unbounded_channel::<Task>();

    let mut pending_queue = get_tasks();
    let mut processed_buffer = Vec::<Task>::new();

    spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10));

        loop {
            interval.tick().await;

            while let Some(task) = pending_queue.pop_front() {
                if !task.can_be_started() {
                    pending_queue.push_back(task);
                } else {
                    let ctx = tx.clone();
                    // note: issues when `some_computation` is async
                    spawn_blocking(move || some_computation(task, ctx));
                }

                println!("{:?}", pending_queue);
            }
        }
    });

    while let Some(res) = rx.recv().await {
        processed_buffer.push(res);
        println!("{:?}", processed_buffer);
    }
}

fn some_computation(mut task: Task, tx: UnboundedSender<Task>) {
    let block_period = Duration::from_secs(5);

    let res = task.set_as_running();
    match res {
        Ok(_) => tracing::info!("started task {:?}", task),
        Err(_) => tracing::error!("failed to run {:?}", task),
    }

    sleep(block_period);
    task.set_as_completed().unwrap_or(());
    tracing::info!("completed task {:?}", task);

    tx.send(task)
        .unwrap_or(tracing::error!("failed sending task"));
}

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
