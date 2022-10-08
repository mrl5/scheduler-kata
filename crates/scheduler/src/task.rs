use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::fmt;
use std::fs::remove_dir_all;
use std::io::Write;
use std::{
    fs::{create_dir_all, File},
    hash::{Hash, Hasher},
    path::PathBuf,
};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

pub const FS_PERSIST_PATH: [&str; 2] = ["/tmp", "mrl5-scheduler-kata"];

pub fn flush() -> anyhow::Result<()> {
    let path: PathBuf = FS_PERSIST_PATH.iter().collect();

    tracing::info!("flushing {:?} ...", path);
    remove_dir_all(&path)?;
    create_dir_all(&path)?;
    Ok(())
}

#[derive(Eq, Debug, Clone, Copy, Serialize)]
pub struct Task {
    // note: normally I'd use uuidv4 but it was fun to learn about Ksuid :D
    // btw I have minor feedback regarding using this library
    // update: changed to Uuid because of Serde serialize -> looks like this could be improved in
    // Ksuid library :) and looks like good first issue ;)
    pub id: Uuid,
    pub task_type: TaskType,
    pub scheduled_for: OffsetDateTime,
    status: TaskStatus,
    fs_persist_path: &'static [&'static str],
}

#[derive(Eq, Debug, Hash, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum TaskType {
    TypeA,
    TypeB,
    TypeC,
}

#[derive(Eq, Debug, Hash, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Error, Debug, Clone)]
pub enum TaskError {
    #[error("This task can't be re-run")]
    TaskReRun,
    #[error("Can't change to status")]
    TaskFlow, // note: I had issues passing enum here as a parameter
    #[error("Task persist error")] // todo: inject task_id here
    TaskPersist,
}

impl Task {
    pub fn new(task_type: TaskType, scheduled_for: OffsetDateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            scheduled_for,
            status: TaskStatus::Pending,
            fs_persist_path: &FS_PERSIST_PATH,
        }
    }

    pub fn persist_blocking(&self) -> anyhow::Result<()> {
        let path: PathBuf = self.fs_persist_path.iter().collect();
        let f = &path.join(format!("{}.json", self.id));

        create_dir_all(&path)?;
        let mut f = File::create(f)?;
        writeln!(f, "{}", self)?;

        Ok(())
    }

    pub fn can_be_started(&self) -> bool {
        self.status == TaskStatus::Pending && self.scheduled_for < OffsetDateTime::now_utc()
    }

    pub fn get_status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn set_as_running(&mut self) -> anyhow::Result<(), TaskError> {
        if self.can_be_started() {
            self.status = TaskStatus::Running;
            match self.persist_blocking() {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("{}", e);
                    Err(TaskError::TaskPersist)
                }
            }
        } else {
            Err(TaskError::TaskReRun)
        }
    }

    pub fn set_as_completed(&mut self) -> Result<(), TaskError> {
        if self.get_status() == &TaskStatus::Running {
            self.status = TaskStatus::Completed;
            match self.persist_blocking() {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("{}", e);
                    Err(TaskError::TaskPersist)
                }
            }
        } else {
            Err(TaskError::TaskReRun)
        }
    }

    pub fn set_as_failed(&mut self) -> Result<(), TaskError> {
        if self.get_status() == &TaskStatus::Running {
            self.status = TaskStatus::Failed;
            match self.persist_blocking() {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("{}", e);
                    Err(TaskError::TaskPersist)
                }
            }
        } else {
            Err(TaskError::TaskFlow)
        }
    }
}

impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.to_string().hash(state);
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            to_string(&self).unwrap_or_else(|_| "{}".to_owned())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_instantiate_different_tasks() {
        let t1 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
        let t2 = Task::new(TaskType::TypeB, OffsetDateTime::now_utc());
        let t3 = Task::new(TaskType::TypeC, OffsetDateTime::now_utc());

        assert_ne!(t1.id.to_string(), t2.id.to_string());
        assert_ne!(t1.id.to_string(), t3.id.to_string());
        assert_ne!(t2.id.to_string(), t3.id.to_string());
    }

    #[test]
    fn it_should_know_if_can_be_started() {
        let delayed = OffsetDateTime::now_utc();
        let delayed = delayed.replace_year(delayed.year() + 1).unwrap();

        let t1 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
        let mut t2 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
        let mut t3 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
        let t4 = Task::new(TaskType::TypeB, delayed);

        assert!(t1.can_be_started());

        assert!(t2.set_as_running().is_ok());
        assert!(!t2.can_be_started());
        assert!(t2.set_as_completed().is_ok());
        assert!(t2.set_as_running().is_err());
        assert!(!t2.can_be_started());

        assert!(t3.set_as_running().is_ok());
        assert!(t3.set_as_failed().is_ok());
        assert!(t3.set_as_completed().is_err());
        assert!(t3.set_as_running().is_err());
        assert!(!t3.can_be_started());

        assert!(!t4.can_be_started());
    }
}
