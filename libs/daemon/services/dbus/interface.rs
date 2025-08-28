use o324_dbus::{dto, O324ServiceInterface};
use typed_builder::TypedBuilder;
use zbus::{fdo, interface};

use crate::services::{
    storage_bridge::{DbOperation, StorageBridgeService},
    task::{error::TaskServiceError, TaskService},
};

#[derive(TypedBuilder)]
pub struct O324Service {
    task_service: TaskService,
    storage_bridge_service: StorageBridgeService,
}

#[interface(name = "org.o324.Service1")]
impl O324ServiceInterface for O324Service {
    async fn start_new_task(&self, input: dto::StartTaskInputDto) -> fdo::Result<dto::TaskDto> {
        self.task_service
            .start_new_task(input.into())
            .await
            .map(|task| task.into())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn stop_current_task(&self) -> fdo::Result<Option<dto::TaskDto>> {
        self.task_service
            .stop_current_task()
            .await
            .map(|x| x.map(|t| t.into()))
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn cancel_current_task(&self) -> fdo::Result<Option<dto::TaskDto>> {
        self.task_service
            .cancel_current_task()
            .await
            .map(|x| x.map(|t| t.into()))
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn delete_task(&self, task_id: String) -> fdo::Result<Option<dto::TaskDto>> {
        self.task_service
            .delete_task(task_id)
            .await
            .map(|x| x.map(|t| t.into()))
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn get_task_by_id(&self, task_id: String) -> fdo::Result<Option<dto::TaskDto>> {
        self.task_service
            .get_task(task_id)
            .await
            .map(|x| x.map(|t| t.into()))
            .or_else(|e| match e {
                TaskServiceError::Default(e) => Err(fdo::Error::Failed(e.to_string())),
                TaskServiceError::RefError(items) => todo!(),
            })
    }

    async fn edit_task(
        &self,
        task_ref_str: String,
        update: dto::TaskUpdateDto,
    ) -> fdo::Result<dto::TaskDto> {
        self.task_service
            .edit_task(task_ref_str.as_str().into(), update.into())
            .await
            .map(|t| t.into())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn list_last_tasks(&self, count: u64) -> fdo::Result<Vec<dto::TaskDto>> {
        self.task_service
            .list_last_tasks(count)
            .await
            .map(|tasks| tasks.into_iter().map(|t| t.into()).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> fdo::Result<Vec<dto::TaskDto>> {
        self.task_service
            .list_task_range(start_timestamp, end_timestamp)
            .await
            .map(|tasks| tasks.into_iter().map(|t| t.into()).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn db_query(
        &self,
        operation: dto::DbOperationDto,
    ) -> fdo::Result<dto::DbResultDtoPacked> {
        let internal_operation =
            DbOperation::try_from(operation).map_err(fdo::Error::InvalidArgs)?;

        let res: dto::DbResultDto = self
            .storage_bridge_service
            .db_query(internal_operation)
            .map_err(|e| fdo::Error::Failed(e.to_string()))?
            .into();

        Ok(res.pack())
    }

    async fn ping(&self) -> fdo::Result<String> {
        Ok("pong".into())
    }
}
