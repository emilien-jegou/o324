#![feature(map_entry_replace)]

use erased_serde::Serialize;
use serde::de::DeserializeOwned;

pub use git_document_db_macros as prelude;

pub(crate) mod client;
pub(crate) mod connection;
pub(crate) mod connection_config;
pub mod document_parser;
pub(crate) mod errors;
pub(crate) mod lock_manager;
pub(crate) mod query_runner;
pub(crate) mod shared_query_runner;
pub(crate) mod sync_runner;
pub(crate) mod transaction;

mod utils {
    pub(crate) mod advisory_lock;
    pub(crate) mod files;
    pub(crate) mod system_lock;
    #[cfg(test)]
    pub(crate) mod test_utilities;
    pub(crate) mod thread_lock;
}

mod git_actions {
    #[cfg(target_os = "linux")]
    mod fetch;
    #[cfg(target_os = "linux")]
    mod init;
    #[cfg(target_os = "linux")]
    mod push;
    #[cfg(target_os = "linux")]
    pub mod rebase;
    #[cfg(target_os = "linux")]
    mod reset_to_head;
    #[cfg(target_os = "linux")]
    mod stage_and_commit_changes;

    #[cfg(target_os = "linux")]
    pub use fetch::fetch;
    #[cfg(target_os = "linux")]
    pub use init::init;
    #[cfg(target_os = "linux")]
    pub use push::push;
    #[cfg(target_os = "linux")]
    pub use rebase::rebase_current_branch;
    #[cfg(target_os = "linux")]
    pub use reset_to_head::reset_to_head;
    #[cfg(target_os = "linux")]
    pub use stage_and_commit_changes::stage_and_commit_changes;
}

pub trait Document: Serialize + DeserializeOwned + std::fmt::Debug {
    fn get_document_id(&self) -> String;
    fn set_document_id(&mut self, v: &str);
}

pub use client::Client;
pub use connection::Connection;
pub use connection_config::ConnectionConfig;
pub use errors::{StoreError, StoreResult};
pub use query_runner::{IQueryRunner, QueryRunner};
pub use shared_query_runner::SharedQueryRunner;
pub use sync_runner::{DocumentRef, SyncConflict, SyncRunner};
pub use transaction::Transaction;
