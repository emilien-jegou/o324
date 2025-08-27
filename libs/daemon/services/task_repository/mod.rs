use crate::{
    config::Config,
    core::{
        storage::{DbOperation, DbResult, Storage},
        utils::{self, generate_random_id},
    },
    entities::task::{Task, TaskId, TaskKey, TaskUpdate},
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use wrap_builder::wrap_builder;

#[wrap_builder(Arc)]
pub struct TaskRepository {
    pub config: Config,
    pub storage: Storage,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StartTaskInput {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
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

impl TaskRepositoryInner {
    /// Starts a new task. If another task is currently running, it will be stopped.
    pub async fn start_new_task(
        &self,
        input: StartTaskInput,
    ) -> eyre::Result<(Task, Vec<TaskAction>)> {
        let current_timestamp = utils::unix_now();
        let mut task_actions = Vec::new();

        let task = self.storage.write(|qr| {
            // If a task is already running, stop it first by setting its end time.
            if let Some(mut current) = qr
                .get()
                .secondary::<Task>(TaskKey::end, None as Option<u64>)?
            {
                current.end = Some(current_timestamp);
                qr.upsert(current.clone())?;
                task_actions.push(TaskAction::Upsert(current));
            }

            // Create and start the new task with a random ID.
            let task_id = generate_random_id(7);
            let new_task = Task::builder()
                .id(task_id.clone())
                .task_name(input.task_name)
                .project(input.project)
                .computer_name(self.config.core.computer_name.clone())
                .tags(input.tags)
                .start(current_timestamp)
                .end(None)
                .build(); // try_build() already computes the hash
            qr.upsert(new_task.clone())?;
            task_actions.push(TaskAction::Upsert(new_task.clone()));
            Ok(new_task)
        })?;

        Ok((task, task_actions))
    }

    /// Stops the currently running task by setting its end time.
    pub async fn stop_current_task(&self) -> eyre::Result<(Option<Task>, Vec<TaskAction>)> {
        let current_timestamp = utils::unix_now();
        let mut task_actions = Vec::new();

        let stopped_task = self.storage.write(|qr| {
            // Find the currently running task by querying for a task with `end: None`.
            if let Some(mut current_task) = qr
                .get()
                .secondary::<Task>(TaskKey::end, None as Option<u64>)?
            {
                // Update its end time and recompute the hash.
                current_task.end = Some(current_timestamp);

                // Save it and record the action.
                qr.upsert(current_task.clone())?;
                task_actions.push(TaskAction::Upsert(current_task.clone()));
                Ok(Some(current_task))
            } else {
                Ok(None)
            }
        })?;

        Ok((stopped_task, task_actions))
    }

    /// Cancels and deletes the currently running task.
    pub async fn cancel_current_task(&self) -> eyre::Result<(Option<Task>, Vec<TaskAction>)> {
        let mut task_actions = Vec::new();

        let canceled_task = self.storage.write(|qr| {
            // Find the currently running task.
            if let Some(current_task) = qr
                .get()
                .secondary::<Task>(TaskKey::end, None as Option<u64>)?
            {
                let task_id = current_task.id.clone();
                // Remove the task entity itself.
                qr.remove(current_task.clone())?;
                task_actions.push(TaskAction::Delete(task_id));
                Ok(Some(current_task))
            } else {
                Ok(None)
            }
        })?;

        Ok((canceled_task, task_actions))
    }

    /// Deletes a task by its specific ID.
    pub async fn delete_task(
        &self,
        task_id: String,
    ) -> eyre::Result<(Option<Task>, Vec<TaskAction>)> {
        let mut task_actions = Vec::new();

        let deleted_task = self.storage.write(|qr| {
            if let Some(task_to_delete) = qr.get().primary::<Task>(task_id.clone())? {
                qr.remove(task_to_delete.clone())?;
                task_actions.push(TaskAction::Delete(task_id));
                Ok(Some(task_to_delete))
            } else {
                Ok(None)
            }
        })?;

        Ok((deleted_task, task_actions))
    }

    /// Edits an existing task, identified by its ID or as the "current" task.
    pub async fn edit_task(
        &self,
        task_ref: TaskRef,
        update_task: TaskUpdate,
    ) -> eyre::Result<(Task, Vec<TaskAction>)> {
        let mut task_actions = Vec::new();
        let current_timestamp = utils::unix_now();

        let task = self.storage.write(|qr| {
            let task_id = match task_ref {
                TaskRef::Current => {
                    qr.get()
                        .secondary::<Task>(TaskKey::end, None as Option<u64>)?
                        .ok_or_else(|| eyre::eyre!("No current task to edit"))?
                        .id
                }
                TaskRef::Id(id) => id,
            };

            let original_task = qr
                .get()
                .primary::<Task>(task_id.clone())?
                .ok_or_else(|| eyre::eyre!("Task with ID '{}' not found", &task_id))?;

            let new_task = update_task.merge_with_task(&original_task);

            let was_running = original_task.end.is_none();
            let is_now_running = new_task.end.is_none();

            // If this edit makes a task running (i.e., resumes it),
            // we must first stop any other task that is currently running.
            if is_now_running && !was_running {
                if let Some(mut other_current_task) = qr
                    .get()
                    .secondary::<Task>(TaskKey::end, None as Option<u64>)?
                {
                    if other_current_task.id != new_task.id {
                        other_current_task.end = Some(current_timestamp);
                        qr.upsert(other_current_task.clone())?;
                        task_actions.push(TaskAction::Upsert(other_current_task));
                    }
                }
            }

            qr.upsert(new_task.clone())?;
            task_actions.push(TaskAction::Upsert(new_task.clone()));

            Ok(new_task)
        })?;

        Ok((task, task_actions))
    }

    pub async fn get_task_by_id(&self, task_id: TaskId) -> eyre::Result<Option<Task>> {
        self.storage
            .read(|qr| Ok(qr.get().primary::<Task>(task_id)?))
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

    pub async fn db_query(&self, operation: DbOperation) -> eyre::Result<DbResult> {
        self.storage.db_query(operation)
    }
}
