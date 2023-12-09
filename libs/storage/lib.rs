use std::ops::Deref;

use serde::de::DeserializeOwned;

pub mod storage {
    #[cfg(feature = "git")]
    pub mod git;
    pub mod in_memory;
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

pub trait Storage {
    fn debug_message(&self);
}

#[derive(Clone, Debug)]
pub enum BuiltinStorageType {
    #[cfg(feature = "git")]
    Git,
    InMemory,
}
