use erased_serde::Serialize;
use serde::de::DeserializeOwned;

pub use git_document_db_macros as prelude;

pub(crate) mod client;
pub(crate) mod client_config;
pub(crate) mod errors;
pub(crate) mod transaction;
pub mod document_parser;

mod utils {
    pub(crate) mod files;
    #[cfg(test)]
    pub(crate) mod test_utilities;
    pub(crate) mod system_lock;
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

pub trait Document: Serialize + DeserializeOwned {
    fn get_document_id(&self) -> String;
    fn set_document_id(&mut self, v: &str);
}

pub use client::{Client, SyncConflict};
pub use client_config::ClientConfig;
pub use errors::{StoreError, StoreResult};
