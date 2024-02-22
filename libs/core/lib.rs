use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use o324_storage::{patronus::Setter, StorageBox, Task, TaskUpdate};

pub mod config;
mod load;
mod utils;

pub use load::{load, load_core};
use ulid::Ulid;

pub struct Core {
    storage: StorageBox,
    /// Ok - found | Err - not found with error reason
    found_config_file: Result<(), eyre::Error>,
}

pub struct StartTaskInput {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum TaskRef {
    /// Reference the task currently running
    Current,
    /// Reference a task by it's ID
    Id(String),
}

impl FromStr for TaskRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "current" => TaskRef::Current,
            id => TaskRef::Id(id.to_string()),
        };
        Ok(m)
    }
}

// TODO: prevent invalid character in task name (e.g. '#', '-')
impl Core {
    pub async fn initialize(&self) -> eyre::Result<()> {
        self.storage.init().await?;
        Ok(())
    }

    pub async fn start_new_task(&self, input: StartTaskInput) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;
        let current_timestamp = utils::unix_now();
        let current = self.storage.get_current_task_id().await?;

        if let Some(task_id) = current {
            self.storage
                .update_task(task_id, TaskUpdate::default().set_end(current_timestamp))
                .await?;
        }

        let task_id = Ulid::new().to_string();
        self.storage
            .create_task(Task {
                ulid: task_id.clone(),
                task_name: input.task_name,
                project: input.project,
                tags: input.tags,
                start: current_timestamp,
                end: None,
            })
            .await?;

        self.storage.set_current_task_id(Some(task_id)).await?;

        lock.release().await?;
        Ok(())
    }

    pub async fn stop_current_task(&self) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;
        let current = self.storage.get_current_task_id().await?;
        let current_timestamp = utils::unix_now();

        if let Some(task_id) = current {
            self.storage
                .update_task(task_id, TaskUpdate::default().set_end(current_timestamp))
                .await?;
            self.storage.set_current_task_id(None).await?;
        }

        lock.release().await?;
        Ok(())
    }

    pub async fn cancel_current_task(&self) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;
        let current = self.storage.get_current_task_id().await?;

        if let Some(task_id) = current {
            self.storage.delete_task(task_id).await?;
            self.storage.set_current_task_id(None).await?;
        }

        lock.release().await?;
        Ok(())
    }

    pub async fn delete_task(&self, task_id: String) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;
        self.storage.delete_task(task_id).await?;
        lock.release().await?;
        Ok(())
    }

    pub async fn edit_task(&self, task_ref: TaskRef, update_task: TaskUpdate) -> eyre::Result<()> {
        if let Setter::Set(_) = update_task.ulid {
            return Err(eyre::eyre!("Updating the task id directly is not allowed"));
        }
        let mut lock = self.storage.try_lock().await?;

        let current_task_id = self.storage.get_current_task_id().await?;

        let task_id = match task_ref {
            TaskRef::Current => current_task_id
                .clone()
                .ok_or_else(|| eyre::eyre!("No task currently running"))?,
            TaskRef::Id(v) => v,
        };

        if let Setter::Set(end) = update_task.end {
            // Some preprocessing need to be done if the user decided to update the end date of
            // a task:
            // - if the end date of the current task is set then we need to stop the task
            // - if the end date is removed from a closed task then we should raise an error
            match end {
                Some(_) => {
                    if Some(task_id.clone()) == current_task_id.clone() {
                        self.storage.set_current_task_id(None).await?;
                    }
                }
                None => {
                    Err(eyre::eyre!("you cannot set the end value of a task to none if it has already been ended"))?;
                }
            };
        }

        // TODO: verify that we cannot set an end date superior to the start date
        if let Setter::Set(start) = update_task.start {
            // If the task changed start date we need to re-assign it's id;
            // sorting the task by id should sort them by start date, we want to keep this
            // mechanism as currently the git storage depends on it for finding tasks efficiently.

            let prev_task = self.storage.get_task(task_id.clone()).await?;

            let system_time = UNIX_EPOCH
                .checked_add(Duration::from_secs(start))
                .ok_or_else(|| eyre::eyre!("Couldn't parse task start timestamp"))?;

            if system_time > SystemTime::now() {
                return Err(eyre::eyre!("Invalid start date is in the future"));
            }

            let new_task_id = Ulid::from_datetime(system_time).to_string();
            let mut new_task = update_task.merge_with_task(&prev_task);

            new_task.ulid = new_task_id.clone();

            self.storage.create_task(new_task.clone()).await?;

            self.storage.delete_task(task_id).await?;

            if new_task.end.is_none() {
                self.storage.set_current_task_id(Some(new_task_id)).await?;
            } else if prev_task.end.is_none() {
                self.storage.set_current_task_id(None).await?;
            }

            return Ok(());
        }

        self.storage.update_task(task_id, update_task).await?;
        lock.release().await?;
        Ok(())
    }

    pub async fn list_last_tasks(&self, count: u64) -> eyre::Result<Vec<Task>> {
        let mut lock = self.storage.try_lock().await?;
        let tasks = self.storage.list_last_tasks(count).await?;
        lock.release().await?;
        Ok(tasks)
    }

    /// Returned an ordered list of task between given dates
    pub async fn list_task_range(
        &self,
        _start_timestamp: u64,
        _end_timestamp: u64,
    ) -> eyre::Result<Vec<Task>> {
        todo!();
        //let mut lock = self.storage.try_lock().await?;
        //self.storage.delete_task(task_id).await?;
        //lock.release().await?;
        //Ok(())
    }

    pub fn get_inner_storage(&self) -> &StorageBox {
        &self.storage
    }

    pub fn has_found_config_file(&self) -> &Result<(), eyre::Error> {
        &self.found_config_file
    }
}
