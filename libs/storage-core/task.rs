use patronus::patronus;
use serde_derive::{Deserialize, Serialize};

pub type TaskId = String;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[patronus(
    name = "TaskUpdate",
    derives = "Default, Debug, Deserialize, PartialEq, Clone"
)]
pub struct Task {
    pub ulid: TaskId,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub start: u64,
    pub end: Option<u64>,
    pub __version: u32,
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
        Task {
            ulid: self.ulid.unwrap_or(task.ulid.clone()),
            task_name: self.task_name.unwrap_or(task.task_name.clone()),
            project: self.project.unwrap_or(task.project.clone()),
            tags: self.tags.unwrap_or(task.tags.clone()),
            start: self.start.unwrap_or(task.start),
            end: self.end.unwrap_or(task.end),
            __version: self.__version.unwrap_or(task.__version) + 1,
        }
    }

    // This is an helper method
    pub fn get_ulid(&self) -> eyre::Result<String> {
        match &self.ulid {
            Some(ulid) => Ok(ulid.clone()),
            None => Err(eyre::eyre!("task ulid is a required field")),
        }
    }
}
