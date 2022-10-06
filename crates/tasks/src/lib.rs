use svix_ksuid::{Ksuid, KsuidLike};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Eq, Debug, PartialEq)]
pub struct Task {
    pub id: Ksuid,
    pub task_type: TaskType,
    pub execution_time: OffsetDateTime,
    status: TaskStatus,
}

#[derive(Eq, Debug, Hash, PartialEq)]
pub enum TaskType {
    TypeA,
    TypeB,
    TypeC,
}

#[derive(Eq, Debug, Hash, PartialEq)]
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
}

impl Task {
    pub fn new(task_type: TaskType, execution_time: OffsetDateTime) -> Self {
        Self {
            id: Ksuid::new(None, None),
            task_type,
            execution_time,
            status: TaskStatus::Pending,
        }
    }

    pub fn can_be_started(&self) -> bool {
        self.status == TaskStatus::Pending
    }

    pub fn get_status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn set_as_running(&mut self) -> Result<(), TaskError> {
        if self.can_be_started() {
            self.status = TaskStatus::Running;
            Ok(())
        } else {
            Err(TaskError::TaskReRun)
        }
    }

    pub fn set_as_completed(&mut self) -> Result<(), TaskError> {
        if self.get_status() == &TaskStatus::Running {
            self.status = TaskStatus::Completed;
            Ok(())
        } else {
            Err(TaskError::TaskReRun)
        }
    }

    pub fn set_as_failed(&mut self) -> Result<(), TaskError> {
        if self.get_status() == &TaskStatus::Running {
            self.status = TaskStatus::Failed;
            Ok(())
        } else {
            Err(TaskError::TaskFlow)
        }
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
        let t1 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
        let mut t2 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());
        let mut t3 = Task::new(TaskType::TypeA, OffsetDateTime::now_utc());

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
    }
}
