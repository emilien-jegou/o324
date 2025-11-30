use crate::{
    core::{
        storage::Storage,
        utils::{self, generate_random_id},
    },
    entities::task::{Task, TaskId, TaskKey, TaskUpdate},
};
use std::sync::Arc;
use wrap_builder::wrap_builder;

use defs::{StartTaskInput, TaskAction, TaskRef};

pub mod defs;

#[wrap_builder(Arc)]
pub struct TaskRepository {
    pub computer_name: String,
    pub storage: Storage,
}

impl TaskRepositoryInner {
    pub async fn start_new_task(
        &self,
        input: StartTaskInput,
    ) -> eyre::Result<(Task, Vec<TaskAction>)> {
        let current_timestamp = utils::unix_now_ms();
        let mut task_actions = Vec::new();

        // Determine effective start and end times
        let (start_ts, end_ts) = match input.at {
            Some(at) => (at.start, at.end),
            None => (current_timestamp, None),
        };

        let task = self.storage.write_txn(|qr| {
            // If the new task is "open" (running), we check if we need to stop the currently running task.
            // We do not stop the current task if we are just inserting a historic/completed task (end_ts is Some).
            if end_ts.is_none() {
                if let Some(mut current) = qr
                    .get()
                    .secondary::<Task>(TaskKey::end, None as Option<u64>)?
                {
                    // Only stop the current task if the new task starts AFTER (or same time as)
                    // the current task's start time.
                    // If start_ts < current.start, we are inserting an open task in the past
                    // (backfilling) and should not interrupt the task currently running "now".
                    if start_ts >= current.start {
                        current.end = Some(start_ts);
                        qr.upsert(current.clone())?;
                        task_actions.push(TaskAction::Upsert(current));
                    }
                }
            }

            // Create and start the new task with a random ID.
            let task_id = generate_random_id(7);
            let new_task = Task::builder()
                .id(task_id.clone())
                .task_name(input.task_name)
                .project(input.project)
                .computer_name(self.computer_name.clone())
                .tags(input.tags)
                .start(start_ts)
                .end(end_ts)
                .build(); // try_build() already computes the hash

            qr.upsert(new_task.clone())?;
            task_actions.push(TaskAction::Upsert(new_task.clone()));
            Ok(new_task)
        })?;

        Ok((task, task_actions))
    }

    /// Stops the currently running task by setting its end time.
    pub async fn stop_current_task(&self) -> eyre::Result<(Option<Task>, Vec<TaskAction>)> {
        let current_timestamp = utils::unix_now_ms();
        let mut task_actions = Vec::new();

        let stopped_task = self.storage.write_txn(|qr| {
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

        let canceled_task = self.storage.write_txn(|qr| {
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

        let deleted_task = self.storage.write_txn(|qr| {
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
        let current_timestamp = utils::unix_now_ms();

        let task = self.storage.write_txn(|qr| {
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

    pub async fn match_prefix(&self, task_id_prefix: TaskId) -> eyre::Result<Vec<Task>> {
        self.storage.read_txn(|qr| {
            let tasks = qr
                .scan()
                .primary::<Task>()?
                .start_with(task_id_prefix)?
                .collect::<Result<Vec<Task>, _>>()?;

            Ok(tasks)
        })
    }

    pub async fn get_task_by_id(&self, task_id: TaskId) -> eyre::Result<Option<Task>> {
        self.storage
            .read_txn(|qr| Ok(qr.get().primary::<Task>(task_id)?))
    }

    pub async fn list_last_tasks(&self, offset: u64, count: u64) -> eyre::Result<Vec<Task>> {
        self.storage.read_txn(|qr| {
            let tasks = qr
                .scan()
                .secondary::<Task>(TaskKey::start)?
                .all()?
                .rev()
                .skip(offset as usize)
                .take(count as usize)
                .collect::<Result<Vec<Task>, _>>()?;

            Ok(tasks)
        })
    }

    pub async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> eyre::Result<Vec<Task>> {
        self.storage.read_txn(|qr| {
            let filtered_tasks = qr
                .scan()
                .secondary::<Task>(TaskKey::start)?
                .range(start_timestamp..end_timestamp)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(filtered_tasks)
        })
    }
}
