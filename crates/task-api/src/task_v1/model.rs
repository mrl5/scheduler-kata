use chrono::{DateTime, Utc};
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

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskId {
    pub id: Uuid,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, JsonSchema)]
pub struct Task {
    pub id: Uuid,
    pub typ: String,
    pub state: Option<String>,
    pub created_at: DateTime<Utc>,
    pub not_before: Option<DateTime<Utc>>,
    pub inactive_since: Option<DateTime<Utc>>,
}
