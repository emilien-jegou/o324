use patronus::patronus;
use serde_derive::{Deserialize, Serialize};

pub type TaskId = String;

#[derive(Clone, Serialize, Deserialize)]
#[patronus("TaskUpdate")]
pub struct Task {
    pub ulid: TaskId,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub start: u64,
    pub end: Option<u64>,
}

impl TaskUpdate {
    pub fn merge_with_task(self, task: &Task) -> Task {
        Task {
            ulid: self.ulid.or(task.ulid.clone()),
            task_name: self.task_name.or(task.task_name.clone()),
            project: self.project.or(task.project.clone()),
            tags: self.tags.or(task.tags.clone()),
            start: self.start.or(task.start),
            end: self.end.or(task.end),
        }
    }
}
