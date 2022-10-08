use crate::task::Task;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub fn some_computation(
    mut task: Task,
    receive_tx: UnboundedSender<Uuid>,
    release_tx: UnboundedSender<Task>,
) -> anyhow::Result<()> {
    receive_tx.send(task.id)?;

    let block_period = Duration::from_secs(5);
    let id = task.id.to_string();
    let task_type = task.task_type.to_owned();

    let res = task.set_as_running();
    match res {
        Ok(_) => tracing::info!("started task {:?}: {}", task_type, id),
        Err(_) => {
            tracing::error!("failed to run task {:?}: {}", task_type, id);
            return Ok(release_tx.send(task)?);
        }
    }

    sleep(block_period);

    match task.set_as_completed() {
        Ok(_) => tracing::info!("completed task {:?}: {}", task_type, id),
        Err(_) => tracing::error!("unable to mark task {} as completed", id),
    };

    Ok(release_tx.send(task)?)
}
