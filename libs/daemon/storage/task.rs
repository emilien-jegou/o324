use std::hash::{DefaultHasher, Hash, Hasher};

use derive_builder::Builder;
use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use patronus::patronus;
use serde::{Deserialize, Serialize};

pub type TaskId = String;

#[native_model(id = 1, version = 1)]
#[native_db]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Builder)]
#[patronus(
    name = "TaskUpdate",
    derives = "Default, Debug, Deserialize, PartialEq, Clone"
)]
#[builder(build_fn(skip))]
pub struct Task {
    #[primary_key]
    pub id: String,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    #[secondary_key]
    pub start: u64,
    pub computer_name: String,
    // needs to be unique for indexing reason
    #[secondary_key(unique)]
    pub end: Option<u64>,
    #[builder(setter(skip))]
    __hash: u64,
}

impl TaskBuilder {
    pub fn try_build(&self) -> eyre::Result<Task> {
        let mut task = Task {
            id: self
                .id
                .clone()
                .ok_or_else(|| eyre::eyre!("field id is not set"))?,
            task_name: self
                .task_name
                .clone()
                .ok_or_else(|| eyre::eyre!("field task_name is not set"))?,
            project: self.project.clone().unwrap_or_default(),
            computer_name: self.computer_name.clone().unwrap_or_default(),
            tags: self.tags.clone().unwrap_or_default(),
            start: self
                .start
                .ok_or_else(|| eyre::eyre!("field start is not set"))?,
            end: self.end.unwrap_or_default(),
            __hash: 0,
        };
        task.compute_new_hash();
        Ok(task)
    }
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
    pub fn compute_new_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        self.__hash = hasher.finish();
    }

    pub fn get_hash(&self) -> u64 {
        self.__hash
    }
}

impl TaskUpdate {
    pub fn from_task_diff(left: &Task, right: &Task) -> eyre::Result<TaskUpdate> {
        if left.id != right.id {
            return Err(eyre::eyre!("diff between task with different id"));
        }

        let mut res = TaskUpdate::default().set_id(left.id.clone());

        if left.task_name != right.task_name {
            res = res.set_task_name(right.task_name.clone());
        }

        if left.project != right.project {
            res = res.set_project(right.project.clone());
        }

        if left.tags != right.tags {
            res = res.set_tags(right.tags.to_vec());
        }

        if left.start != right.start {
            res = res.set_start(right.start);
        }

        if left.end != right.end {
            res = res.set_end(right.end);
        }

        Ok(res)
    }

    pub fn merge_with_task(self, task: &Task) -> Task {
        let mut task = Task {
            id: self.id.unwrap_or(task.id.clone()),
            task_name: self.task_name.unwrap_or(task.task_name.clone()),
            project: self.project.unwrap_or(task.project.clone()),
            computer_name: self.computer_name.unwrap_or(task.computer_name.clone()),
            tags: self.tags.unwrap_or(task.tags.clone()),
            start: self.start.unwrap_or(task.start),
            end: self.end.unwrap_or(task.end),
            __hash: 0,
        };
        task.compute_new_hash();
        task
    }

    // This is an helper method
    pub fn get_id(&self) -> eyre::Result<String> {
        match &self.id {
            Some(id) => Ok(id.clone()),
            None => Err(eyre::eyre!("task id is a required field")),
        }
    }
}
