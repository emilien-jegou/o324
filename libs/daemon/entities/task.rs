use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use patronus::patronus;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use typed_builder::TypedBuilder;

pub type TaskId = String;

#[native_model(id = 1, version = 1)]
#[native_db]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TypedBuilder)]
#[patronus(
    name = "TaskUpdate",
    derives = "Default, Debug, Deserialize, PartialEq, Clone"
)]
//#[builder(build_fn(skip))]
pub struct Task {
    #[primary_key]
    pub id: String,
    pub task_name: String,
    #[builder(default = None)]
    pub project: Option<String>,
    #[builder(default = Vec::new())]
    pub tags: Vec<String>,
    #[secondary_key]
    pub start: u64,
    pub computer_name: String,
    // needs to be unique for indexing reason
    #[secondary_key(unique)]
    #[builder(default = None)]
    pub end: Option<u64>,
}

impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.task_name.hash(state);
        self.project.hash(state);
        self.tags.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl Task {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl TaskUpdate {
    pub fn merge_with_task(self, task: &Task) -> Task {
        Task {
            id: self.id.unwrap_or(task.id.clone()),
            task_name: self.task_name.unwrap_or(task.task_name.clone()),
            project: self.project.unwrap_or(task.project.clone()),
            computer_name: self.computer_name.unwrap_or(task.computer_name.clone()),
            tags: self.tags.unwrap_or(task.tags.clone()),
            start: self.start.unwrap_or(task.start),
            end: self.end.unwrap_or(task.end),
        }
    }
}
