//! A D-Bus client for the o324 service.
//!
//! This client connects to the `org.o324.Service` on the session bus
//! and demonstrates calling its methods.
use zbus::fdo;
use crate::dto;

#[zbus::proxy(
    interface = "org.o324.Service1",
    default_service = "org.o324.Service",
    default_path = "/org/o324/Service"
)]
pub trait O324Service {
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
}
