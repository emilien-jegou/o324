pub mod dto;
#[cfg(feature = "proxy")]
pub mod proxy;

pub use zbus;
pub use zvariant;

/// A macro to define the methods of the o324 service interface.
/// This ensures that the service contract and the proxy trait are identical.
#[macro_export]
macro_rules! define_o324_service_interface_methods {
    () => {
        async fn start_new_task(
            &self,
            input: dto::StartTaskInputDto,
        ) -> fdo::Result<Vec<dto::TaskActionDto>>;
        async fn stop_current_task(&self) -> fdo::Result<Vec<dto::TaskActionDto>>;
        async fn cancel_current_task(&self) -> fdo::Result<Vec<dto::TaskActionDto>>;
        async fn delete_task(&self, task_id: String) -> fdo::Result<Vec<dto::TaskActionDto>>;
        async fn edit_task(
            &self,
            task_ref_str: String,
            update: dto::TaskUpdateDto,
        ) -> fdo::Result<Vec<dto::TaskActionDto>>;
        async fn list_last_tasks(&self, count: u64) -> fdo::Result<Vec<dto::TaskDto>>;
        async fn list_task_range(
            &self,
            start_timestamp: u64,
            end_timestamp: u64,
        ) -> fdo::Result<Vec<dto::TaskDto>>;
        async fn ping(&self) -> fdo::Result<String>;
        async fn get_task_by_id(&self, task_id: String) -> fdo::Result<Option<dto::TaskDto>>;
        async fn db_query(&self, operation: dto::DbOperationDto) -> fdo::Result<dto::DbResultDto>;
    };
}
