use crate::{Storage, StorageConfig};
use serde_derive::Deserialize;

pub struct GitStorage {
    config: GitStorageConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct GitStorageConfig {}

impl StorageConfig for GitStorageConfig {
    type Storage = GitStorage;

    fn to_storage(self) -> Box<dyn Storage> {
        Box::new(GitStorage::new(self))
    }
}

impl Storage for GitStorage {
    fn debug_message(&self) {
        println!("Git storage");
        println!("config: {:?}", self.config);
    }
}

impl GitStorage {
    pub fn new(config: GitStorageConfig) -> Self {
        GitStorage { config }
    }
}
