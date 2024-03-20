#![feature(trait_upcasting)]
#![feature(concat_idents)]

mod config;
pub(crate) mod module;
mod storage;

pub(crate) mod providers {
    pub mod git_transaction_provider;
}

mod task_actions {
    pub mod task_change;
    pub mod repair_unique_current_task;
    pub mod resolve_task_conflict;
}

pub(crate) mod models {
    pub mod metadata_document;
    pub mod task_document;
}

pub(crate) mod managers {
    pub mod metadata_document_manager;
    pub mod task_document_manager;
}

pub(crate) mod services {
    pub mod git_service;
    pub mod metadata_service;
    pub mod storage_sync_service;
    pub mod task_service;
}

pub(crate) mod utils {
    pub mod files;
    #[cfg(test)]
    pub mod test_utilities;
    pub mod ulid;
}

pub use config::GitStorageConfig;
pub use storage::GitStorage;
