use std::hash::{DefaultHasher, Hash, Hasher};

use derive_builder::Builder;
use patronus::patronus;
use serde_derive::{Deserialize, Serialize};

pub type TaskId = String;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Builder)]
#[patronus(
    name = "TaskUpdate",
    derives = "Default, Debug, Deserialize, PartialEq, Clone"
)]
#[builder(build_fn(skip))]
pub struct Task {
    pub ulid: TaskId,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub start: u64,
    pub computer_name: String,
    pub end: Option<u64>,
    #[builder(setter(skip))]
    __hash: u64,
}

impl TaskBuilder {
    pub fn try_build(&self) -> eyre::Result<Task> {
        let mut task = Task {
            ulid: self
                .ulid
                .clone()
                .ok_or_else(|| eyre::eyre!("field ulid is not set"))?,
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
        self.ulid.hash(state);
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
        if left.ulid != right.ulid {
            return Err(eyre::eyre!("diff between task with different id"));
        }

        let mut res = TaskUpdate::default().set_ulid(left.ulid.clone());

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
            ulid: self.ulid.unwrap_or(task.ulid.clone()),
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
    pub fn get_ulid(&self) -> eyre::Result<String> {
        match &self.ulid {
            Some(ulid) => Ok(ulid.clone()),
            None => Err(eyre::eyre!("task ulid is a required field")),
        }
    }
}
