use crate::{
    git_actions, lock_manager::LockManager, query_runner::IQueryRunner,
    utils::advisory_lock::SystemLockType, Connection, Document, QueryRunner, SharedQueryRunner,
    StoreError, StoreResult,
};
use lazy_regex::Regex;

pub struct Transaction<'a> {
    query_runner: QueryRunner<'a>,
    connection: &'a Connection,
    lock_manager: LockManager,
}

/// The transaction ensure that readers are blocked during write operation
/// NB: document level locking is currently not implemented, the whole storage get locked on
/// write operation, I may revisit this in the future
impl<'a> Transaction<'a> {
    pub fn new(connection: &'a Connection, query_runner: QueryRunner<'a>) -> Self {
        Self {
            lock_manager: LockManager::default(),
            connection,
            query_runner,
        }
    }

    fn try_lock_store(&self, lock_type: SystemLockType) -> StoreResult<()> {
        self.lock_manager.try_lock_store(lock_type)?;
        Ok(())
    }

    fn try_lock_document(&self, document_id: &str, lock_type: SystemLockType) -> StoreResult<()> {
        self.lock_manager
            .try_lock_document(document_id, lock_type)?;
        Ok(())
    }

    pub fn commit_on_change(&self, action: &str) -> eyre::Result<()> {
        let repository = self
            .connection
            .repository
            .lock()
            .map_err(StoreError::system_error)?;
        let rg = format!("*\\.{}", self.connection.document_parser.file_extension());
        git_actions::stage_and_commit_changes(&repository, &format!("Hey - {action}"), &[&rg])?;
        Ok(())
    }

    pub fn release(&self) -> eyre::Result<()> {
        self.commit_on_change("transaction")?;
        self.lock_manager.release_all()?;
        Ok(())
    }

    pub fn abort(&self) -> eyre::Result<()> {
        let repository = self
            .connection
            .repository
            .lock()
            .map_err(StoreError::system_error)?;
        let rg = format!("*\\.{}", self.connection.document_parser.file_extension());
        git_actions::reset_to_head(&repository, &[&rg])?;
        self.lock_manager.release_all()?;
        Ok(())
    }
}

impl<'a> IQueryRunner<'a> for Transaction<'a> {
    fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>> {
        // Keep lock order as is for correct unlocking:
        self.try_lock_document(document_id, SystemLockType::Exclusive)?;
        self.try_lock_store(SystemLockType::Exclusive)?;
        self.query_runner.get(document_id)
    }

    fn get_document_list(&self) -> StoreResult<Vec<String>> {
        self.try_lock_store(SystemLockType::Exclusive)?;
        self.query_runner.get_document_list()
    }

    fn find_matching<T: Document>(&self, document_id_regex: &Regex) -> StoreResult<Vec<T>> {
        self.try_lock_store(SystemLockType::Exclusive)?;
        self.query_runner.find_matching(document_id_regex)
    }

    fn save<T: Document>(&self, document: &T) -> StoreResult<()> {
        // Keep lock order as is for correct unlocking:
        // TODO: document lock for write could be an atomic update
        self.try_lock_document(&document.get_document_id(), SystemLockType::Exclusive)?;
        self.try_lock_store(SystemLockType::Exclusive)?;
        self.query_runner.save(document)
    }

    fn to_shared_runner(&'a self) -> SharedQueryRunner<'a> {
        SharedQueryRunner::Transaction(self)
    }
}
