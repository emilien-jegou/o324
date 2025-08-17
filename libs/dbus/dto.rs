use serde::{Deserialize, Serialize};
use zvariant::{Optional, Type};

type TaskId = String;

#[derive(Type, Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct TaskDto {
    pub id: TaskId,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub computer_name: String,
    pub start: u64,
    pub end: Option<u64>,
    pub __hash: u64,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct TaskUpdateDto {
    pub task_name: Optional<String>,
    pub project: Optional<Option<String>>,
    pub tags: Optional<Vec<String>>,
    pub start: Optional<u64>,
    pub end: Optional<Option<u64>>,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct StartTaskInputDto {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Type, Debug, Deserialize, Serialize)]
pub enum TaskActionType {
    Upsert,
    Delete,
}

// This is not an enum due to zbus limitations
#[derive(Type, Serialize, Deserialize, Debug)]
pub struct TaskActionDto {
    pub action_type: TaskActionType,
    pub upsert_action: Optional<TaskDto>,
    pub delete_action: Optional<TaskId>,
}
