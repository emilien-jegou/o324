use eyre::Error;
use o324_dbus::{dto, O324ServiceInterface};
use zbus::{fdo, interface};

use crate::{
    core::storage::DbOperation,
    services::task::{TaskRef, TaskService},
};

pub struct O324Service {
    pub task_service: TaskService,
}

#[interface(name = "org.o324.Service1")]
impl O324ServiceInterface for O324Service {
    async fn start_new_task(&self, input: dto::StartTaskInputDto) -> fdo::Result<dto::TaskDto> {
        let (task, _) = self
            .task_service
            .start_new_task(input.into())
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let prefix = self
            .task_service
            .prefix_index
            .find_shortest_unique_prefix(&task.id)
            .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

        Ok(task.into_dto(prefix))
    }

    async fn stop_current_task(&self) -> fdo::Result<Option<dto::TaskDto>> {
        let (task, _) = self
            .task_service
            .stop_current_task()
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let task_dto = if let Some(t) = task {
            let prefix = self
                .task_service
                .prefix_index
                .find_shortest_unique_prefix(&t.id)
                .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

            Some(t.into_dto(prefix))
        } else {
            None
        };

        Ok(task_dto)
    }

    async fn cancel_current_task(&self) -> fdo::Result<Option<dto::TaskDto>> {
        let (task, _) = self
            .task_service
            .cancel_current_task()
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let task_dto = if let Some(t) = task {
            let prefix = self
                .task_service
                .prefix_index
                .find_shortest_unique_prefix(&t.id)
                .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

            Some(t.into_dto(prefix))
        } else {
            None
        };

        Ok(task_dto)
    }

    async fn delete_task(&self, task_id: String) -> fdo::Result<Option<dto::TaskDto>> {
        let (task, _) = self
            .task_service
            .delete_task(task_id)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let task_dto = if let Some(t) = task {
            let prefix = self
                .task_service
                .prefix_index
                .find_shortest_unique_prefix(&t.id)
                .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

            Some(t.into_dto(prefix))
        } else {
            None
        };

        Ok(task_dto)
    }

    async fn get_task_by_id(&self, task_id: String) -> fdo::Result<Option<dto::TaskDto>> {
        let maybe_task = self
            .task_service
            .get_task_by_id(task_id)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let maybe_dto_result: fdo::Result<Option<dto::TaskDto>> = maybe_task
            .map(|task| {
                let prefix = self
                    .task_service
                    .prefix_index
                    .find_shortest_unique_prefix(&task.id)
                    .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

                Ok(task.into_dto(prefix))
            })
            .transpose();

        maybe_dto_result
    }

    async fn edit_task(
        &self,
        task_ref_str: String,
        update: dto::TaskUpdateDto,
    ) -> fdo::Result<dto::TaskDto> {
        let task_ref = task_ref_str
            .parse::<TaskRef>() // <-- The fix is here
            .map_err(|e| fdo::Error::InvalidArgs(e.to_string()))?;

        let (task, _) = self
            .task_service
            .edit_task(task_ref, update.into())
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let prefix = self
            .task_service
            .prefix_index
            .find_shortest_unique_prefix(&task.id)
            .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

        Ok(task.into_dto(prefix))
    }

    async fn list_last_tasks(&self, count: u64) -> fdo::Result<Vec<dto::TaskDto>> {
        let tasks = self
            .task_service
            .list_last_tasks(count)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        tasks
            .into_iter()
            .map(|task| {
                let prefix = self
                    .task_service
                    .prefix_index
                    .find_shortest_unique_prefix(&task.id)
                    .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

                Ok(task.into_dto(prefix))
            })
            .collect()
    }

    async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> fdo::Result<Vec<dto::TaskDto>> {
        let tasks = self
            .task_service
            .list_task_range(start_timestamp, end_timestamp)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        tasks
            .into_iter()
            .map(|task| {
                let prefix = self
                    .task_service
                    .prefix_index
                    .find_shortest_unique_prefix(&task.id)
                    .map_err(|e: Error| fdo::Error::Failed(e.to_string()))?;

                Ok(task.into_dto(prefix))
            })
            .collect()
    }

    async fn db_query(&self, operation: dto::DbOperationDto) -> fdo::Result<dto::DbResultDto> {
        let internal_operation =
            DbOperation::try_from(operation).map_err(fdo::Error::InvalidArgs)?;

        match self.task_service.db_query(internal_operation).await {
            Ok(internal_result) => {
                let result_dto: dto::DbResultDto = internal_result.into();
                Ok(result_dto)
            }
            Err(e) => {
                let error_dto = dto::DbResultDto {
                    result_type: dto::DbResultTypeDto::Error,
                    table_list: None,
                    table_rows: None,
                    error: Some(e.to_string()),
                };
                Ok(error_dto)
            }
        }
    }

    async fn ping(&self) -> fdo::Result<String> {
        Ok("pong".into())
    }
}
