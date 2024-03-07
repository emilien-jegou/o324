#![feature(trait_upcasting)]
#![feature(concat_idents)]

mod config;
pub(crate) mod module;
mod storage;

pub(crate) mod providers {
    pub mod git_transaction_provider;
}

mod git_actions {
    mod fetch;
    mod init;
    mod push;
    pub mod rebase;
    mod stage_and_commit_changes;

    pub use fetch::fetch;
    pub use init::init;
    pub use push::push;
    pub use rebase::rebase_current_branch;
    pub use stage_and_commit_changes::stage_and_commit_changes;
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
    pub mod config_manager;
    pub mod document_storage_manager;
    pub mod file_format_manager;
    pub mod git_manager;
    pub mod git_sync_manager;
    pub mod metadata_document_manager;
    pub mod task_document_manager;
}

pub(crate) mod services {
    pub mod metadata_service;
    pub mod task_service;
}

pub(crate) mod utils {
    pub mod files;
    pub mod system_lock;
    #[cfg(test)]
    pub mod test_utilities;
    pub mod ulid;
}

pub use config::GitStorageConfig;
pub use storage::GitStorage;
