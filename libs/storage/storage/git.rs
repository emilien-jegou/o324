use crate::{
    utils::semaphore::Semaphore, PinFuture, Storage, StorageBox, StorageConfig, Transaction,
    TransactionBox,
};
use serde_derive::Deserialize;

/// Save data as json inside of a git directory
pub struct GitStorage {
    config: GitStorageConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct GitStorageConfig {}

impl StorageConfig for GitStorageConfig {
    type Storage = GitStorage;

    fn to_storage(self) -> StorageBox {
        StorageBox::new(GitStorage::new(self))
    }
}

impl Storage for GitStorage {
    fn debug_message(&self) {
        println!("Git storage");
        println!("config: {:?}", self.config);
    }

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move { Ok(TransactionBox::new(GitTransaction::try_new()?)) })
    }
}

impl GitStorage {
    pub fn new(config: GitStorageConfig) -> Self {
        GitStorage { config }
    }
}

pub struct GitTransaction {
    lock: Semaphore,
}

const GIT_SEMAPHORE_NAME: &str = "3to4-git-transaction-3";

impl GitTransaction {
    pub fn try_new() -> eyre::Result<Self> {
        let mut lock = Semaphore::try_new(GIT_SEMAPHORE_NAME)?;
        lock.try_acquire()?;
        Ok(GitTransaction { lock })
    }
}

impl Transaction for GitTransaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.lock.release()?;
            Ok(())
        })
    }
}

impl Drop for GitTransaction {
    fn drop(&mut self) {
        // TODO: this is somewhat unsafe
        self.lock.release().expect("Couldn't release semaphore");
    }
}
