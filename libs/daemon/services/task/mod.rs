use std::sync::Arc;

use crate::{
    entities::task::{Task, TaskUpdate},
    repositories::{
        task::{
            defs::{StartTaskInput, TaskRef},
            TaskRepository,
        },
        task_prefix::TaskPrefixRepository,
    },
};
use wrap_builder::wrap_builder;

pub mod error;

#[wrap_builder(Arc)]
pub struct TaskService {
    task_repository: TaskRepository,
    task_prefix_repository: TaskPrefixRepository,
}

pub struct TaskWithMeta {
    pub task: Task,
    pub prefix: String,
}

impl TaskServiceInner {
    pub async fn start_new_task(&self, input: StartTaskInput) -> eyre::Result<TaskWithMeta> {
        let (task, _) = self.task_repository.start_new_task(input).await?;

        self.task_prefix_repository
            .add_ids(std::slice::from_ref(&task.id))?;

        let prefix = self
            .task_prefix_repository
            .find_shortest_unique_prefix(&task.id)?;

        Ok(TaskWithMeta { task, prefix })
    }

    pub async fn stop_current_task(&self) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_repository.stop_current_task().await?;

        let task = if let Some(task_inner) = task {
            let prefix = self
                .task_prefix_repository
                .find_shortest_unique_prefix(&task_inner.id)?;

            Some(TaskWithMeta {
                task: task_inner,
                prefix,
            })
        } else {
            None
        };

        Ok(task)
    }

    pub async fn cancel_current_task(&self) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_repository.cancel_current_task().await?;

        let task = if let Some(task_inner) = task {
            let prefix = self
                .task_prefix_repository
                .find_shortest_unique_prefix(&task_inner.id)?;

            Some(TaskWithMeta {
                task: task_inner,
                prefix,
            })
        } else {
            None
        };

        Ok(task)
    }

    pub async fn delete_task(&self, task_id: String) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_repository.delete_task(task_id).await?;

        let task = if let Some(task_inner) = task {
            let prefix = self
                .task_prefix_repository
                .find_shortest_unique_prefix(&task_inner.id)?;

            Some(TaskWithMeta {
                task: task_inner,
                prefix,
            })
        } else {
            None
        };

        Ok(task)
    }

    pub async fn get_task(&self, task_ref: String) -> error::Result<Option<TaskWithMeta>> {
        let task_ids = self.task_prefix_repository.search_by_prefix(&task_ref)?;

        if task_ids.len() > 1 {
            let unique_ids_matchs = task_ids
                .into_iter()
                .filter(|t| t.is_end_of_id)
                .map(|t| t.prefix)
                .collect::<Vec<String>>();
            return Err(error::TaskServiceError::RefError(unique_ids_matchs));
        }

        let maybe_task = self.task_repository.get_task_by_id(task_ref).await?;

        let task: Option<TaskWithMeta> = maybe_task
            .map(|task| -> eyre::Result<TaskWithMeta> {
                let prefix = self
                    .task_prefix_repository
                    .find_shortest_unique_prefix(&task.id)?;

                Ok(TaskWithMeta { task, prefix })
            })
            .transpose()?;

        Ok(task)
    }

    pub async fn edit_task(
        &self,
        task_ref: TaskRef,
        update: TaskUpdate,
    ) -> eyre::Result<TaskWithMeta> {
        let (task, _) = self.task_repository.edit_task(task_ref, update).await?;

        let prefix = self
            .task_prefix_repository
            .find_shortest_unique_prefix(&task.id)?;

        Ok(TaskWithMeta { task, prefix })
    }

    pub async fn list_last_tasks(&self, count: u64) -> eyre::Result<Vec<TaskWithMeta>> {
        let tasks = self.task_repository.list_last_tasks(count).await?;

        tasks
            .into_iter()
            .map(|task| {
                let prefix = self
                    .task_prefix_repository
                    .find_shortest_unique_prefix(&task.id)?;

                Ok(TaskWithMeta { task, prefix })
            })
            .collect()
    }

    pub async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> eyre::Result<Vec<TaskWithMeta>> {
        let tasks = self
            .task_repository
            .list_task_range(start_timestamp, end_timestamp)
            .await?;

        tasks
            .into_iter()
            .map(|task| {
                let prefix = self
                    .task_prefix_repository
                    .find_shortest_unique_prefix(&task.id)?;

                Ok(TaskWithMeta { task, prefix })
            })
            .collect()
    }
}
