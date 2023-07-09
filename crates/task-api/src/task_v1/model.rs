use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Display, JsonSchema)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    TypeA,
    TypeB,
    TypeC,
}

#[derive(Serialize, JsonSchema)]
pub struct TaskId {
    pub id: Uuid,
}
