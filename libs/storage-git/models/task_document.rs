use git_document_db::{prelude::*, Document};
use o324_storage_core::{Task, TaskId};
use std::collections::BTreeMap;

#[derive(Debug, Default, Document)]
pub struct TaskDocument {
    #[document(id)]
    pub id: String,
    /// Ordered map of TaskId to Task
    pub tasks: BTreeMap<TaskId, Task>,
}
