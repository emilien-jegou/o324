use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use serde::de::DeserializeOwned;

pub mod storage {
    #[cfg(feature = "git")]
    pub mod git;
    pub mod in_memory;
}

pub mod utils {
    #[cfg(feature = "git")]
    pub(crate) mod semaphore;
}

pub struct StorageBox(Box<dyn Storage>);

impl Deref for StorageBox {
    type Target = dyn Storage;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl StorageBox {
    fn new(storage: impl Storage + 'static) -> Self {
        Self(Box::new(storage))
    }
}

pub trait StorageConfig: DeserializeOwned + Default {
    type Storage: Storage;

    fn to_storage(self) -> StorageBox;
}

pub struct TransactionBox(Box<dyn Transaction>);

impl Deref for TransactionBox {
    type Target = dyn Transaction;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for TransactionBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl TransactionBox {
    fn new(storage: impl Transaction + 'static) -> Self {
        Self(Box::new(storage))
    }
}

type PinFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Transaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>>;
}

pub trait Storage: Sync {
    fn debug_message(&self);

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>>;

    //let txn = self.storage.try_lock().await?;
    //if self.storage.has_active_task().await? == true {
    //self.storage.add_new_task(Task {
    //txn.commit().await?;
}

#[derive(Clone, Debug)]
pub enum BuiltinStorageType {
    #[cfg(feature = "git")]
    Git,
    InMemory,
}
