use o324_storage_core::{Task, TaskId};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Default)]
pub struct TaskDocument {
    /// Ordered map of TaskId to Task
    pub tasks: BTreeMap<TaskId, Task>,
}
