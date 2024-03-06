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
}

pub(crate) mod utils {
    pub mod files;
    pub mod semaphore;
    #[cfg(test)]
    pub mod test_utilities;
}


use std::time::{Duration, UNIX_EPOCH};

pub use config::GitStorageConfig;
use o324_storage_core::Task;
pub use storage::GitStorage;
use ulid::Ulid;

pub type GitDailyDocument = Vec<Task>;

pub(crate) fn ulid_from_timestamp(unix_timestamp: u64) -> eyre::Result<String> {
    let system_time = UNIX_EPOCH
        .checked_add(Duration::from_secs(unix_timestamp))
        .ok_or_else(|| eyre::eyre!("Couldn't parse timestamp"))?;

    Ok(Ulid::from_datetime(system_time).to_string())
}

// Create a ulid from a timestamp and fill the second part of the timestamp with the $c character
pub(crate) fn ulid_from_timestamp_with_overwrite(
    unix_timestamp: u64,
    c: char,
) -> eyre::Result<String> {
    let mut ulid = ulid_from_timestamp(unix_timestamp)?;
    ulid.replace_range(10..26, &c.to_string().repeat(16));
    Ok(ulid)
}
