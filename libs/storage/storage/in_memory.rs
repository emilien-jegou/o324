use crate::{PinFuture, Storage, StorageBox, StorageConfig, TransactionBox};
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

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move { todo!() })
    }
}

impl InMemoryStorage {
    pub fn new(config: InMemoryStorageConfig) -> Self {
        InMemoryStorage { config }
    }
}
