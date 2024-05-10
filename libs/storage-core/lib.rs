use std::{future::Future, pin::Pin};

mod storage;
mod storage_config;
mod task;
mod transaction;

pub use storage::{
    LockType, Storage, TaskAction, StorageClient, StorageContainer, StorageFn, StorageTransaction,
};
pub use storage_config::StorageConfig;
pub use task::{Task, TaskBuilder, TaskId, TaskUpdate};
pub use transaction::{Transaction, TransactionContainer};

pub type PinFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
