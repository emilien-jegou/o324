use std::sync::Arc;

use crate::{
    core::storage::{DbOperation, DbResult},
    entities::task::{Task, TaskUpdate},
    services::{
        task_prefix_repository::TaskPrefixRepository,
        task_repository::{StartTaskInput, TaskRef, TaskRepository},
    },
};
use wrap_builder::wrap_builder;

#[wrap_builder(Arc)]
pub struct TaskManagerService {
    task_service: TaskRepository,
    task_prefix_repository: TaskPrefixRepository,
}

pub struct TaskWithMeta {
    pub task: Task,
    pub prefix: String,
}

impl TaskManagerServiceInner {
    pub async fn start_new_task(&self, input: StartTaskInput) -> eyre::Result<TaskWithMeta> {
        let (task, _) = self.task_service.start_new_task(input).await?;

        self.task_prefix_repository.add_ids(&[task.id.clone()])?;

        let prefix = self
            .task_prefix_repository
            .find_shortest_unique_prefix(&task.id)?;

        Ok(TaskWithMeta { task, prefix })
    }

    pub async fn stop_current_task(&self) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_service.stop_current_task().await?;

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
        let (task, _) = self.task_service.cancel_current_task().await?;

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
        let (task, _) = self.task_service.delete_task(task_id).await?;

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

    pub async fn get_task_by_id(&self, task_id: String) -> eyre::Result<Option<TaskWithMeta>> {
        let maybe_task = self.task_service.get_task_by_id(task_id).await?;

        let maybe_result: eyre::Result<Option<TaskWithMeta>> = maybe_task
            .map(|task| {
                let prefix = self
                    .task_prefix_repository
                    .find_shortest_unique_prefix(&task.id)?;

                Ok(TaskWithMeta { task, prefix })
            })
            .transpose();

        maybe_result
    }

    pub async fn edit_task(
        &self,
        task_ref: TaskRef,
        update: TaskUpdate,
    ) -> eyre::Result<TaskWithMeta> {
        //let task_ref = task_ref_str
        //    .parse::<TaskRef>() // <-- The fix is here
        //    .map_err(|e| eyre::Error::InvalidArgs(e.to_string()))?;

        let (task, _) = self.task_service.edit_task(task_ref, update).await?;

        let prefix = self
            .task_prefix_repository
            .find_shortest_unique_prefix(&task.id)?;

        Ok(TaskWithMeta { task, prefix })
    }

    pub async fn list_last_tasks(&self, count: u64) -> eyre::Result<Vec<TaskWithMeta>> {
        let tasks = self.task_service.list_last_tasks(count).await?;

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
            .task_service
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

    // TODO: this should be moved out
    pub async fn db_query(&self, operation: DbOperation) -> eyre::Result<DbResult> {
        self.task_service.db_query(operation).await
    }
}
