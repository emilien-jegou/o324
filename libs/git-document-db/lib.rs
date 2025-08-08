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
    pub(crate) mod files;
    pub(crate) mod system_lock;
    pub(crate) mod advisory_lock;
    pub(crate) mod thread_lock;
    #[cfg(test)]
    pub(crate) mod test_utilities;
}

mod git_actions {
    mod fetch;
    mod init;
    mod push;
    pub mod rebase;
    mod stage_and_commit_changes;
    mod reset_to_head;

    pub use fetch::fetch;
    pub use init::init;
    pub use push::push;
    pub use rebase::rebase_current_branch;
    pub use stage_and_commit_changes::stage_and_commit_changes;
    pub use reset_to_head::reset_to_head;
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
