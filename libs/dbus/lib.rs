pub mod dto;
#[cfg(feature = "proxy")]
pub mod proxy;

pub use zbus;
use zbus::fdo;
pub use zvariant;

pub trait O324ServiceInterface {
    fn start_new_task(
        &self,
        input: dto::StartTaskInputDto,
    ) -> impl std::future::Future<Output = fdo::Result<dto::TaskDto>>;
    fn stop_current_task(
        &self,
    ) -> impl std::future::Future<Output = fdo::Result<Option<dto::TaskDto>>>;
    fn cancel_current_task(
        &self,
    ) -> impl std::future::Future<Output = fdo::Result<Option<dto::TaskDto>>>;
    fn delete_task(
        &self,
        task_id: String,
    ) -> impl std::future::Future<Output = fdo::Result<Option<dto::TaskDto>>>;
    fn edit_task(
        &self,
        task_ref_str: String,
        update: dto::TaskUpdateDto,
    ) -> impl std::future::Future<Output = fdo::Result<dto::TaskDto>>;
    fn list_last_tasks(
        &self,
        offset: u64,
        count: u64,
    ) -> impl std::future::Future<Output = fdo::Result<Vec<dto::TaskDto>>>;
    fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> impl std::future::Future<Output = fdo::Result<Vec<dto::TaskDto>>>;
    fn ping(&self) -> impl std::future::Future<Output = fdo::Result<String>>;
    fn get_task_by_prefix(
        &self,
        task_ref: String,
    ) -> impl std::future::Future<Output = fdo::Result<dto::TaskByPrefixDtoPacked>>;
    fn db_query(
        &self,
        operation: dto::DbOperationDto,
    ) -> impl std::future::Future<Output = fdo::Result<dto::DbResultDtoPacked>>;
}
