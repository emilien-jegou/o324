use super::{
    task::{Task, TaskId, TaskUpdate},
    transaction::TransactionBox,
};
use crate::PinFuture;
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
#[deref(forward)]
#[deref_mut(forward)]
pub struct StorageBox(pub Box<dyn Storage>);

impl StorageBox {
    pub fn new(storage: impl Storage + 'static) -> Self {
        Self(Box::new(storage))
    }
}

pub trait Storage: Sync {
    fn debug_message(&self);

    fn init(&self, config: &o324_config::CoreConfig) -> PinFuture<eyre::Result<()>>;
    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>>;

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
    fn update_task(&self, task_id: String, updated_task: TaskUpdate)
        -> PinFuture<eyre::Result<()>>;

    // Synchronize with external storage
    fn synchronize(&self) -> PinFuture<eyre::Result<()>>;

    // Delete a task by id
    fn delete_task(&self, task_id: String) -> PinFuture<eyre::Result<()>>;

    // Get the active task id
    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>>;

    // Set the active task id
    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>>;
}
