use std::sync::Arc;

use crate::{
    entities::task::{Task, TaskUpdate},
    repositories::{
        project_color::ProjectColorRepository,
        task::{
            defs::{StartTaskInput, TaskRef},
            TaskRepository,
        },
        task_prefix::TaskPrefixRepository,
    },
};
use wrap_builder::wrap_builder;

#[wrap_builder(Arc)]
pub struct TaskService {
    task_repository: TaskRepository,
    task_prefix_repository: TaskPrefixRepository,
    project_color_repository: ProjectColorRepository,
}

pub struct TaskWithMeta {
    pub task: Task,
    pub prefix: String,
    pub project_color_hue: Option<u32>,
}

#[allow(dead_code)]
impl TaskServiceInner {
    async fn task_with_meta(&self, task: Task) -> eyre::Result<TaskWithMeta> {
        let prefix = self
            .task_prefix_repository
            .find_shortest_unique_prefix(&task.id)?;

        let project_color_hue = match task.project.as_ref() {
            Some(project) => Some(self.project_color_repository.get(project).await?),
            None => None,
        };

        Ok(TaskWithMeta {
            task,
            prefix,
            project_color_hue,
        })
    }

    async fn tasks_with_meta(&self, tasks: Vec<Task>) -> eyre::Result<Vec<TaskWithMeta>> {
        let colors = self
            .project_color_repository
            .get_many(
                tasks
                    .iter()
                    .filter_map(|task| task.project.as_deref())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            )
            .await?;

        let mut acc = Vec::new();

        for task in tasks {
            let prefix = self
                .task_prefix_repository
                .find_shortest_unique_prefix(&task.id)?;

            let project_color_hue = task.project.as_ref().map(|p| *colors.get(p).unwrap());

            acc.push(TaskWithMeta {
                prefix,
                project_color_hue,
                task,
            });
        }

        Ok(acc)
    }

    pub async fn start_new_task(&self, input: StartTaskInput) -> eyre::Result<TaskWithMeta> {
        let (task, _) = self.task_repository.start_new_task(input).await?;

        self.task_prefix_repository
            .add_ids(std::slice::from_ref(&task.id))?;

        self.task_with_meta(task).await
    }

    pub async fn stop_current_task(&self) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_repository.stop_current_task().await?;

        let task = if let Some(task_inner) = task {
            Some(self.task_with_meta(task_inner).await?)
        } else {
            None
        };

        Ok(task)
    }

    pub async fn cancel_current_task(&self) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_repository.cancel_current_task().await?;

        let task = if let Some(task_inner) = task {
            Some(self.task_with_meta(task_inner).await?)
        } else {
            None
        };

        Ok(task)
    }

    pub async fn delete_task(&self, task_id: String) -> eyre::Result<Option<TaskWithMeta>> {
        let (task, _) = self.task_repository.delete_task(task_id).await?;

        let task = if let Some(task_inner) = task {
            Some(self.task_with_meta(task_inner).await?)
        } else {
            None
        };

        Ok(task)
    }

    pub async fn get_task(&self, task_ref: String) -> eyre::Result<Option<TaskWithMeta>> {
        let maybe_task = self.task_repository.get_task_by_id(task_ref).await?;

        let task = if let Some(task_inner) = maybe_task {
            Some(self.task_with_meta(task_inner).await?)
        } else {
            None
        };

        Ok(task)
    }

    pub async fn match_prefix(&self, task_ref: String) -> eyre::Result<Vec<TaskWithMeta>> {
        let tasks = self.task_repository.match_prefix(task_ref).await?;

        self.tasks_with_meta(tasks).await
    }

    pub async fn edit_task(
        &self,
        task_ref: TaskRef,
        update: TaskUpdate,
    ) -> eyre::Result<TaskWithMeta> {
        let (task, _) = self.task_repository.edit_task(task_ref, update).await?;

        self.task_with_meta(task).await
    }

    pub async fn list_last_tasks(
        &self,
        offset: u64,
        count: u64,
    ) -> eyre::Result<Vec<TaskWithMeta>> {
        let tasks = self.task_repository.list_last_tasks(offset, count).await?;
        self.tasks_with_meta(tasks).await
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

        self.tasks_with_meta(tasks).await
    }
}
