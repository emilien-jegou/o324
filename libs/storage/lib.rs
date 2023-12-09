use serde::de::DeserializeOwned;

pub mod storage {
    #[cfg(feature = "git")]
    pub mod git;
    pub mod in_memory;
}

pub trait StorageConfig: DeserializeOwned + Default {
    type Storage: Storage;

    fn to_storage(self) -> Box<dyn Storage>;
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
