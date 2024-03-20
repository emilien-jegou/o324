use std::sync::Arc;

use git_document_db::{Client, QueryRunner, StoreResult, SyncConflict, Transaction};
use teloc::Dependency;

#[derive(Dependency, Clone)]
pub struct GitService {
    connection: Arc<git_document_db::Connection>,
}

impl<'a> GitService {
    pub fn sync<F>(&self, callback: F) -> StoreResult<()>
    where
        F: FnMut(&QueryRunner<'_>, &mut Vec<SyncConflict>) -> eyre::Result<()>,
    {
        self.connection.sync(callback)
    }

    pub fn start_transaction(&'a self) -> Transaction<'a> {
        self.connection.transaction()
    }

    pub fn get_client(&'a self) -> Client<'a> {
        self.connection.client()
    }
}
