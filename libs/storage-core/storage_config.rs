use serde::de::DeserializeOwned;

use super::storage::{Storage, StorageContainer};

pub trait StorageConfig: DeserializeOwned + Default {
    type Storage: Storage;

    fn try_into_storage(self) -> eyre::Result<StorageContainer>;
}
