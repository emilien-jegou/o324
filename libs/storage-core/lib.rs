use std::{future::Future, pin::Pin};

mod storage;
mod storage_config;
mod task;
mod transaction;

pub use storage::{Storage, StorageBox};
pub use storage_config::StorageConfig;
pub use task::{Task, TaskId, TaskUpdate};
pub use transaction::{Transaction, TransactionBox};

pub type PinFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
