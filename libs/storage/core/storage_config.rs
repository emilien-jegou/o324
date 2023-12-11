use serde::de::DeserializeOwned;

use super::storage::{Storage, StorageBox};

pub trait StorageConfig: DeserializeOwned + Default {
    type Storage: Storage;

    fn to_storage(self) -> StorageBox;
}
