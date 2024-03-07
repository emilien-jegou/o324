#![feature(trait_upcasting)]

mod config;
mod git_synchronize;
mod storage;
mod task_action;
pub(crate) mod module;

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

pub(crate) mod models {
    pub mod metadata_document;
    pub mod task_document;
}

pub(crate) mod managers {
    pub mod git_manager;
    pub mod metadata_manager;
    pub mod task_manager;
    pub mod model_manager;
}

pub(crate) mod utils {
    pub mod files;
    pub mod semaphore;
    pub mod ulid;
    #[cfg(test)]
    pub mod test_utilities;
}

pub use config::GitStorageConfig;
pub use storage::GitStorage;
