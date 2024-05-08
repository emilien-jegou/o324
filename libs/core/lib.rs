use o324_config::ProfileConfig;
use o324_storage::{StorageContainer, Task, TaskAction, TaskUpdate};
use serde::Deserialize;
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use ulid::Ulid;

mod utils;

pub struct Core {
    pub config: o324_config::CoreConfig,
    pub storage: StorageContainer,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("An error occured when trying to open the configuration file '{0}': {1}")]
    LoadConfigError(String, String),
    #[error("Couldn't find profile '{0}' in configuration file '{1}'")]
    ProfileNotFound(String, String),
    #[error("{0}")]
    ConfigError(String),
}

pub fn load(config_path: &str, profile_name: Option<String>) -> Result<Core, LoadError> {
    let config = o324_config::load(config_path)
        .map_err(|e| LoadError::LoadConfigError(config_path.to_string(), e.to_string()))?;

    let choosen_profile_name =
        profile_name.unwrap_or_else(|| config.core.get_default_profile_name());

    let choosen_profile: &ProfileConfig = config
        .profile
        .iter()
        .find(|(key, _)| *key == &choosen_profile_name)
        .ok_or_else(|| LoadError::ProfileNotFound(choosen_profile_name, config_path.to_string()))?
        .1;

    let storage = o324_storage::load_builtin_storage_from_profile(choosen_profile)
        .map_err(|e| LoadError::ConfigError(e.to_string()))?;

    Ok(Core {
        storage,
        config: config.core,
    })
}

#[derive(Deserialize, Clone, Debug)]
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
        self.storage.init(&self.config).await?;
        Ok(())
    }

    pub async fn start_new_task(&self, input: StartTaskInput) -> eyre::Result<Vec<TaskAction>> {
        let current_timestamp = utils::unix_now();
        let mut qr = self.storage.transaction_2()?;
        let current = qr.get_current_task_id().await?;

        if let Some(task_id) = current {
            qr.update_task(task_id, TaskUpdate::default().set_end(current_timestamp))
                .await?;
        }

        let task_id = Ulid::new().to_string();
        let new_task = Task {
            ulid: task_id.clone(),
            task_name: input.task_name,
            project: input.project,
            tags: input.tags,
            start: current_timestamp,
            end: None,
            __version: 0,
        };
        qr.create_task(new_task.clone()).await?;
        qr.set_current_task_id(Some(task_id)).await?;
        let actions = qr.release()?;
        Ok(actions)
    }

    pub async fn stop_current_task(&self) -> eyre::Result<Vec<TaskAction>> {
        let mut qr = self.storage.transaction_2()?;
        let current = qr.get_current_task_id().await?;
        let current_timestamp = utils::unix_now();

        if let Some(task_id) = &current {
            qr.update_task(
                task_id.clone(),
                TaskUpdate::default().set_end(current_timestamp),
            )
            .await?;
            qr.set_current_task_id(None).await?;
        }

        let actions = qr.release()?;
        Ok(actions)
    }

    pub async fn cancel_current_task(&self) -> eyre::Result<Vec<TaskAction>> {
        let mut qr = self.storage.transaction_2()?;
        let current = qr.get_current_task_id().await?;

        if let Some(task_id) = &current {
            qr.delete_task(task_id.clone()).await?;
            qr.set_current_task_id(None).await?;
        }

        let actions = qr.release()?;
        Ok(actions)
    }

    pub async fn delete_task(&self, task_id: String) -> eyre::Result<Vec<TaskAction>> {
        let mut qr = self.storage.transaction_2()?;
        qr.delete_task(task_id.clone()).await?;
        let current = qr.get_current_task_id().await?;

        // If we delete a task currently running we also need to clean the metadatas
        if current.as_ref() == Some(&task_id) {
            qr.set_current_task_id(None).await?;
        }

        let actions = qr.release()?;
        Ok(actions)
    }

    pub async fn synchronize(&self) -> eyre::Result<()> {
        self.storage.synchronize().await?;
        Ok(())
    }

    pub async fn edit_task(
        &self,
        task_ref: TaskRef,
        update_task: TaskUpdate,
    ) -> eyre::Result<Vec<TaskAction>> {
        if update_task.ulid.is_some() {
            return Err(eyre::eyre!("Updating the task id directly is not allowed"));
        }
        let mut qr = self.storage.transaction_2()?;
        let current_task_id = qr.get_current_task_id().await?;

        let task_id = match task_ref {
            TaskRef::Current => current_task_id.clone().unwrap(),
            TaskRef::Id(v) => v,
        };

        if let Some(end) = update_task.end {
            // Some preprocessing need to be done if the user decided to update the end date of
            // a task:
            // - if the end date of the current task is set, then we need to stop the task
            // - if the end date is removed from a closed task then we should raise an error
            match end {
                Some(_) if Some(&task_id) == current_task_id.as_ref() => {
                    qr.set_current_task_id(None).await?;
                }
                None => {
                    return Err(eyre::eyre!(
                        "you cannot resume a task if it has already been stopped"
                    ));
                }
                _ => {}
            }
        }

        // TODO: verify that we cannot set an end date superior to the start date
        if let Some(start) = update_task.start {
            // If the task changed start date we need to re-assign it's id;
            // sorting the task by id should sort them by start date, we want to keep this
            // mechanism as currently the git storage depends on it for finding tasks efficiently.

            let prev_task = qr.get_task(task_id.clone()).await?;

            let system_time = UNIX_EPOCH
                .checked_add(Duration::from_millis(start))
                .ok_or_else(|| eyre::eyre!("Couldn't parse task start timestamp"))?;

            if system_time > SystemTime::now() {
                return Err(eyre::eyre!("Invalid start date is in the future"));
            }

            let new_task_id = Ulid::from_datetime(system_time).to_string();
            let mut new_task = update_task.merge_with_task(&prev_task);

            new_task.ulid.clone_from(&new_task_id);
            qr.create_task(new_task.clone()).await?;
            qr.delete_task(task_id).await?;

            if new_task.end.is_none() {
                qr.set_current_task_id(Some(new_task_id)).await?;
            } else if prev_task.end.is_none() {
                qr.set_current_task_id(None).await?;
            }
        } else {
            qr.update_task(task_id, update_task).await?;
        }

        let actions = qr.release()?;
        Ok(actions)
    }

    pub async fn list_last_tasks(&self, count: u64) -> eyre::Result<Vec<Task>> {
        let tasks = self.storage.list_last_tasks(count).await?;
        Ok(tasks)
    }

    /// Returned an ordered list of task between given dates
    pub async fn list_task_range(
        &self,
        _start_timestamp: u64,
        _end_timestamp: u64,
    ) -> eyre::Result<Vec<Task>> {
        //let _lock = new_lock(&self.storage, LockType::Shared).await?;
        todo!();
        //let _lock = new_lock(&self.storage).await?;
        //self.storage.delete_task(task_id).await?;
        //lock.release().await?;
        //Ok(())
    }

    pub fn get_inner_storage(&self) -> &StorageContainer {
        &self.storage
    }
}
