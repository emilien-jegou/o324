use crate::{
    git_actions, lock_manager::LockManager, query_runner::IQueryRunner,
    shared_query_runner::SharedQueryRunner, utils::advisory_lock::SystemLockType, Connection,
    Document, QueryRunner, StoreError, StoreResult,
};
use lazy_regex::Regex;

pub struct Client<'a> {
    query_runner: QueryRunner<'a>,
    lock_manager: LockManager,
    connection: &'a Connection,
}

impl<'a> Client<'a> {
    pub fn new(connection: &'a Connection, query_runner: QueryRunner<'a>) -> Self {
        Self {
            lock_manager: LockManager::default(),
            query_runner,
            connection,
        }
    }

    fn commit_on_change(&self, action: &str) -> eyre::Result<()> {
        #[cfg(target_os = "linux")]
        let repository = self
            .connection
            .repository
            .lock()
            .map_err(StoreError::system_error)?;
        let rg = format!("*\\.{}", self.connection.document_parser.file_extension());
        #[cfg(target_os = "linux")]
        git_actions::stage_and_commit_changes(
            &repository,
            &format!("{} - {}", self.connection.name, action),
            &[&rg],
        )?;
        Ok(())
    }
}

impl<'a> IQueryRunner<'a> for Client<'a> {
    fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>> {
        let document_lock = self
            .lock_manager
            .try_lock_document(document_id, SystemLockType::Shared)?;
        let store_lock = self.lock_manager.try_lock_store(SystemLockType::Shared)?;
        let result = self.query_runner.get(document_id);
        document_lock.unlock().map_err(StoreError::lock_error)?;
        store_lock.unlock().map_err(StoreError::lock_error)?;
        result
    }

    fn get_document_list(&self) -> StoreResult<Vec<String>> {
        let store_lock = self.lock_manager.try_lock_store(SystemLockType::Shared)?;
        let result = self.query_runner.get_document_list();
        store_lock.unlock().map_err(StoreError::lock_error)?;
        result
    }

    fn find_matching<T: Document>(&self, document_id_regex: &Regex) -> StoreResult<Vec<T>> {
        let store_lock = self.lock_manager.try_lock_store(SystemLockType::Shared)?;
        let result = self.query_runner.find_matching(document_id_regex);
        store_lock.unlock().map_err(StoreError::lock_error)?;
        result
    }

    fn save<T: Document>(&self, document: &T) -> StoreResult<()> {
        let document_lock = self
            .lock_manager
            .try_lock_document(&document.get_document_id(), SystemLockType::Exclusive)?;
        let store_lock = self
            .lock_manager
            .try_lock_store(SystemLockType::Exclusive)?;
        let result = self.query_runner.save(document);
        self.commit_on_change("save")
            .map_err(StoreError::git_error)?;
        document_lock.unlock().map_err(StoreError::lock_error)?;
        store_lock.unlock().map_err(StoreError::lock_error)?;
        result
    }

    fn to_shared_runner(&'a self) -> SharedQueryRunner<'a> {
        SharedQueryRunner::Client(self)
    }
}
