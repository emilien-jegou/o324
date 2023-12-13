use crate::{
    core::task::{TaskId, TaskUpdate},
    PinFuture, Storage, StorageBox, StorageConfig, Task, TransactionBox,
};
use serde_derive::Deserialize;

/// This storage type is used for testing, data is not persisted to disk but
/// only present in memory
pub struct InMemoryStorage {
    config: InMemoryStorageConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct InMemoryStorageConfig {}

impl StorageConfig for InMemoryStorageConfig {
    type Storage = InMemoryStorage;

    fn to_storage(self) -> StorageBox {
        StorageBox::new(InMemoryStorage::new(self))
    }
}

impl Storage for InMemoryStorage {
    fn debug_message(&self) {
        println!("In memory storage");
        println!("config: {:?}", self.config);
    }

    fn init(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { todo!() })
    }

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move { todo!() })
    }

    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>> {
        Box::pin(async move { todo!() })
    }

    fn set_current_task_id(
        &self,
        _task_id: Option<TaskId>,
    ) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { todo!() })
    }

    fn create_task(&self, _task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { todo!() })
    }

    fn get_task(&self, _task_id: String) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move { todo!() })
    }

    fn list_tasks(
        &self,
        _start_timestamp: u64,
        _end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move { todo!() })
    }

    fn update_task(
        &self,
        _task_id: String,
        _updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { todo!() })
    }

    fn delete_task(&self, _task_id: String) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { todo!() })
    }
}

impl InMemoryStorage {
    pub fn new(config: InMemoryStorageConfig) -> Self {
        InMemoryStorage { config }
    }
}
