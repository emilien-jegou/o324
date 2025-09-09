//! A D-Bus client for the o324 service.
//!
//! This client connects to the `org.o324.Service` on the session bus
//! and demonstrates calling its methods.
use crate::dto;
use zbus::fdo;

#[zbus::proxy(
    interface = "org.o324.Service1",
    default_service = "org.o324.Service",
    default_path = "/org/o324/Service"
)]
pub trait O324Service {
    async fn start_new_task(&self, input: dto::StartTaskInputDto) -> fdo::Result<dto::TaskDto>;
    async fn stop_current_task(&self) -> fdo::Result<Option<dto::TaskDto>>;
    async fn cancel_current_task(&self) -> fdo::Result<Option<dto::TaskDto>>;
    async fn delete_task(&self, task_id: String) -> fdo::Result<Option<dto::TaskDto>>;
    async fn get_task_by_prefix(&self, task_ref: String)
        -> fdo::Result<dto::TaskByPrefixDtoPacked>;
    async fn edit_task(
        &self,
        task_ref_str: String,
        update: dto::TaskUpdateDto,
    ) -> fdo::Result<dto::TaskDto>;
    async fn list_last_tasks(&self, offset: u64, count: u64) -> fdo::Result<Vec<dto::TaskDto>>;
    async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> fdo::Result<Vec<dto::TaskDto>>;
    async fn ping(&self) -> fdo::Result<String>;
    async fn db_query(&self, operation: dto::DbOperationDto)
        -> fdo::Result<dto::DbResultDtoPacked>;
    async fn list_activity_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> fdo::Result<Vec<dto::ActivityDto>>;
}
