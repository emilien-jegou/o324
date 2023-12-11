use std::{future::Future, pin::Pin};

mod core {
    pub(crate) mod storage;
    pub(crate) mod storage_config;
    pub(crate) mod task;
    pub(crate) mod transaction;
}

pub use core::{
    storage::{Storage, StorageBox},
    storage_config::StorageConfig,
    task::Task,
    transaction::{Transaction, TransactionBox},
};

pub mod storage {
    #[cfg(feature = "git")]
    pub mod git;
    pub mod in_memory;
}

pub mod utils {
    #[cfg(feature = "git")]
    pub(crate) mod files;
    #[cfg(feature = "git")]
    pub(crate) mod semaphore;
}

pub(crate) type PinFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone, Debug)]
pub enum BuiltinStorageType {
    #[cfg(feature = "git")]
    Git,
    InMemory,
}
