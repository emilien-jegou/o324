use crate::entities::task::Task;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct StartTaskAt {
    pub start: u64,
    pub end: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct StartTaskInput {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub at: Option<StartTaskAt>,
}

#[derive(Clone, Debug)]
pub enum TaskRef {
    Current,
    Id(String),
}

impl From<&str> for TaskRef {
    fn from(value: &str) -> Self {
        match value {
            "current" => TaskRef::Current,
            id => TaskRef::Id(id.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    Upsert(Task),
    Delete(String),
}
