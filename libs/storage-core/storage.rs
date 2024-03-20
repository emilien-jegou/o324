use super::task::{Task, TaskId, TaskUpdate};
use crate::PinFuture;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Deref, DerefMut)]
#[deref(forward)]
#[deref_mut(forward)]
pub struct StorageContainer(pub Box<dyn Storage>);

impl StorageContainer {
    pub fn new(storage: impl Storage + 'static) -> Self {
        Self(Box::new(storage))
    }
}

pub enum LockType {
    /// Blocks both during exclusive and shared transactions
    Exclusive,
    /// Blocks only during exclusive transactions, allowing concurrent shared transactions
    Shared,
}

pub trait StorageClient: Send + Sync {
    /// Create a new task, if a task was already running then stop it
    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>>;

    // Get a task by id
    fn get_task(&self, task_id: String) -> PinFuture<eyre::Result<Task>>;

    // List all tasks between timestamps
    fn list_tasks_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>>;

    // List last $count tasks
    fn list_last_tasks(&self, count: u64) -> PinFuture<eyre::Result<Vec<Task>>>;

    // Update a task
    fn update_task(
        &self,
        task_id: String,
        updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<Task>>;

    // Delete a task by id
    fn delete_task(&self, task_id: String) -> PinFuture<eyre::Result<()>>;

    // Get the active task id
    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>>;

    // Set the active task id
    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>>;
}

pub type StorageFn = dyn Fn(&dyn StorageClient) -> PinFuture<eyre::Result<()>> + Send;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    // On task creation
    Upsert(Task),
    // On delete
    Delete(TaskId),
}

pub trait StorageTransaction: StorageClient {
    fn release(&mut self) -> eyre::Result<Vec<TaskAction>>;
    fn abort(&mut self) -> eyre::Result<()>;
}

pub trait Storage: StorageClient {
    fn debug_message(&self);

    fn init(&self, config: &o324_config::CoreConfig) -> PinFuture<eyre::Result<()>>;

    fn transaction(
        &self,
        transaction_fn: Box<StorageFn>,
    ) -> PinFuture<eyre::Result<Vec<TaskAction>>>;

    /// Perform a transaction manually by giving direct access to the transaction object.
    /// Clients will have to ensure that the abort or release method is called!
    fn transaction_2(&self) -> eyre::Result<Box<dyn StorageTransaction + '_>>;

    // Synchronize with external storage
    fn synchronize(&self) -> PinFuture<eyre::Result<()>>;
}
