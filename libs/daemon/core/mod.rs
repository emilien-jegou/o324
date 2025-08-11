use native_db::Models;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    config::Config,
    storage::{
        storage::Storage,
        task::{Task, TaskBuilder, TaskKey, TaskUpdate},
    },
};

mod utils;

// This generic struct is the correct architecture.
pub struct Core<'a> {
    pub name: String,
    pub config: Config,
    pub storage: Storage<'a>,
}

impl<'a> Core<'a> {
    pub fn try_new(config: &Config, models: &'a Models) -> eyre::Result<Self> {
        let profile_config = config.get_current_profile()?;

        let mut db_path = profile_config.get_storage_location().clone();
        db_path.push("storage.db");
        // This returns a `StorageContainer<RedbStorage>`
        let storage = Storage::try_new(&db_path, models)
            .map_err(|e| eyre::eyre!("Couldn't initialize storage on path {db_path:?}: {e}"))?;

        Ok(Self {
            name: config.core.computer_name.clone(),
            storage: storage,
            config: config.clone(),
        })
    }
}

impl<'a> std::fmt::Display for Core<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Core({:?})", self.name)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct StartTaskInput {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub computer_name: String,
}

#[derive(Clone, Debug)]
pub enum TaskRef {
    Current,
    Id(String),
}

impl FromStr for TaskRef {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "current" => TaskRef::Current,
            id => TaskRef::Id(id.to_string()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    Upsert(Task),
    Delete(String),
}

impl<'a> Core<'a> {
    pub fn get_loaded_config(&self) -> Config {
        self.config.clone()
    }

    /// Starts a new task. If another task is currently running, it will be stopped.
    pub async fn start_new_task(&self, input: StartTaskInput) -> eyre::Result<Vec<TaskAction>> {
        let current_timestamp = utils::unix_now();
        let mut task_actions = Vec::new();

        self.storage.write(|qr| {
            // If a task is already running, stop it first by setting its end time.
            if let Some(mut current) = qr
                .get()
                .secondary::<Task>(TaskKey::end, None as Option<u64>)?
            {
                current.end = Some(current_timestamp);
                current.compute_new_hash(); // Recompute hash after modification
                qr.upsert(current.clone())?;
                task_actions.push(TaskAction::Upsert(current));
            }

            // Create and start the new task with a random ID.
            let task_id = utils::generate_random_id(7);
            let new_task = TaskBuilder::default()
                .id(task_id.clone())
                .task_name(input.task_name)
                .project(input.project)
                .computer_name(input.computer_name)
                .tags(input.tags)
                .start(current_timestamp)
                .end(None)
                .try_build()?; // try_build() already computes the hash
            qr.upsert(new_task.clone())?;
            task_actions.push(TaskAction::Upsert(new_task));
            Ok(())
        })?;

        Ok(task_actions)
    }

    /// Stops the currently running task by setting its end time.
    pub async fn stop_current_task(&self) -> eyre::Result<Vec<TaskAction>> {
        let current_timestamp = utils::unix_now();
        let mut task_actions = Vec::new();

        self.storage.write(|qr| {
            // Find the currently running task by querying for a task with `end: None`.
            if let Some(mut current_task) = qr
                .get()
                .secondary::<Task>(TaskKey::end, None as Option<u64>)?
            {
                // Update its end time and recompute the hash.
                current_task.end = Some(current_timestamp);
                current_task.compute_new_hash();

                // Save it and record the action.
                qr.upsert(current_task.clone())?;
                task_actions.push(TaskAction::Upsert(current_task));
            }
            Ok(())
        })?;

        Ok(task_actions)
    }

    /// Cancels and deletes the currently running task.
    pub async fn cancel_current_task(&self) -> eyre::Result<Vec<TaskAction>> {
        let mut task_actions = Vec::new();

        self.storage.write(|qr| {
            // Find the currently running task.
            if let Some(current_task) = qr
                .get()
                .secondary::<Task>(TaskKey::end, None as Option<u64>)?
            {
                let task_id = current_task.id.clone();
                // Remove the task entity itself.
                qr.remove(current_task)?;
                task_actions.push(TaskAction::Delete(task_id));
            }
            Ok(())
        })?;

        Ok(task_actions)
    }

    /// Deletes a task by its specific ID.
    pub async fn delete_task(&self, task_id: String) -> eyre::Result<Vec<TaskAction>> {
        let mut task_actions = Vec::new();

        self.storage.write(|qr| {
            // Fetch the task by its primary key to remove it.
            if let Some(task_to_delete) = qr.get().primary::<Task>(task_id.clone())? {
                // Remove the task and record the action.
                qr.remove(task_to_delete)?;
                task_actions.push(TaskAction::Delete(task_id));
            }
            // If the task is not found, do nothing.
            Ok(())
        })?;

        Ok(task_actions)
    }

    /// Edits an existing task, identified by its ID or as the "current" task.
    pub async fn edit_task(
        &self,
        task_ref: TaskRef,
        update_task: TaskUpdate,
    ) -> eyre::Result<Vec<TaskAction>> {
        let mut task_actions = Vec::new();
        let current_timestamp = utils::unix_now();

        self.storage.write(|qr| {
            // 1. Determine the ID of the task to edit.
            let task_id = match task_ref {
                TaskRef::Current => {
                    qr.get()
                        .secondary::<Task>(TaskKey::end, None as Option<u64>)?
                        .ok_or_else(|| eyre::eyre!("No current task to edit"))?
                        .id
                }
                TaskRef::Id(id) => id,
            };

            // 2. Fetch the original task.
            let original_task = qr
                .get()
                .primary::<Task>(task_id.clone())?
                .ok_or_else(|| eyre::eyre!("Task with ID '{}' not found", &task_id))?;

            // 3. Create the new task state by merging the update.
            let new_task = update_task.merge_with_task(&original_task);

            let was_running = original_task.end.is_none();
            let is_now_running = new_task.end.is_none();

            // 4. If this edit makes a task running (i.e., resumes it),
            // we must first stop any other task that is currently running.
            if is_now_running && !was_running {
                if let Some(mut other_current_task) = qr
                    .get()
                    .secondary::<Task>(TaskKey::end, None as Option<u64>)?
                {
                    if other_current_task.id != new_task.id {
                        other_current_task.end = Some(current_timestamp);
                        other_current_task.compute_new_hash();
                        qr.upsert(other_current_task.clone())?;
                        task_actions.push(TaskAction::Upsert(other_current_task));
                    }
                }
            }

            qr.upsert(new_task.clone())?;
            task_actions.push(TaskAction::Upsert(new_task));

            Ok(())
        })?;

        Ok(task_actions)
    }

    pub async fn list_last_tasks(&self, count: u64) -> eyre::Result<Vec<Task>> {
        self.storage.read(|qr| {
            let tasks = qr
                .scan()
                .secondary::<Task>(TaskKey::start)?
                .all()?
                .rev()
                .take(count as usize)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(tasks)
        })
    }

    pub async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> eyre::Result<Vec<Task>> {
        self.storage.read(|qr| {
            let filtered_tasks = qr
                .scan()
                .secondary::<Task>(TaskKey::start)?
                .range(start_timestamp..end_timestamp)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(filtered_tasks)
        })
    }

    pub fn get_inner_storage(&self) -> &Storage {
        &self.storage
    }
}
