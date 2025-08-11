use crate::{
    utils::{advisory_lock::SystemLockType, system_lock::SystemLock},
    StoreError, StoreResult,
};
use dashmap::{
    mapref::{entry::Entry, one::Ref},
    DashMap,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct LockManager {
    manager_name: String,
    locks: DashMap<String, Arc<SystemLock>>,
}

impl<'a> LockManager {
    pub fn new(manager_name: &str) -> Self {
        Self {
            manager_name: format!("git-document-db-lock:{manager_name}"),
            locks: DashMap::new(),
        }
    }

    fn lock(
        &'a self,
        lock_name: &str,
        lock_type: SystemLockType,
    ) -> StoreResult<Ref<'a, String, Arc<SystemLock>>> {
        let safe_lock_name = lock_name.replace("/", "__");
        let entry = self.locks.entry(safe_lock_name.to_owned());
        tracing::trace!("locking '{safe_lock_name:?}' with {lock_type:?} ownership");
        println!("locking '{safe_lock_name:?}' with {lock_type:?} ownership");

        let res = match entry {
            Entry::Occupied(e) => {
                let current = e.into_ref();
                // TODO: proper upgrade/downgrade of the system lock
                current.lock(lock_type).map_err(StoreError::lock_error)?;
                Ok(current)
            }
            Entry::Vacant(e) => {
                let lock = SystemLock::try_new(&safe_lock_name).map_err(StoreError::lock_error)?;
                lock.lock(lock_type.clone())
                    .map_err(StoreError::lock_error)?;
                Ok(e.insert(Arc::new(lock)))
            }
        }?
        .downgrade();

        Ok(res)
    }

    pub fn lock_document(
        &'a self,
        document_name: &str,
        lock_type: SystemLockType,
    ) -> StoreResult<Ref<'a, String, Arc<SystemLock>>> {
        self.lock(
            &format!("{}:{}", self.manager_name, document_name),
            lock_type,
        )
    }

    pub fn lock_store(
        &'a self,
        lock_type: SystemLockType,
    ) -> StoreResult<Ref<'a, String, Arc<SystemLock>>> {
        self.lock(&self.manager_name, lock_type)
    }

    pub fn release_all(&self) -> eyre::Result<()> {
        for it in self.locks.iter() {
            it.value().unlock()?;
        }
        // NIT: this clear locks twice due to Drop implementation
        self.locks.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, Rng};

    use super::*;
    use crate::utils::advisory_lock::SystemLockType;

    fn get_random_manager_name() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }

    #[test]
    fn test_try_lock_document() {
        let manager_name = get_random_manager_name();
        let manager = LockManager::new(&manager_name);
        let lock_type = SystemLockType::Exclusive;

        // Try locking a document
        let result = manager.lock_document("test-lock", lock_type.clone());
        assert!(result.is_ok());

        // Try locking the same document again (should fail because it's already locked)
        tokio::task::spawn({
            let manager = manager.clone();
            async move {
                let result = manager.lock_document("test-lock", lock_type);
                assert!(result.is_ok());
            }
        });

        //std::mem::drop(result);
    }

    #[test]
    fn test_try_lock_store() {
        let manager_name = get_random_manager_name();
        let manager = LockManager::new(&manager_name);
        let lock_type = SystemLockType::Shared;

        // Try locking the store
        let result = manager.lock_store(lock_type.clone());
        assert!(result.is_ok());

        // Try locking the store again (should fail because it's already locked)
        let result = manager.lock_store(lock_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_release_all() {
        let manager_name = get_random_manager_name();
        let manager = LockManager::new(&manager_name);
        let lock_type = SystemLockType::Exclusive;

        // Lock some documents and store
        manager.lock_document("doc1", lock_type.clone()).unwrap();
        manager.lock_document("doc2", lock_type.clone()).unwrap();
        manager.lock_store(lock_type.clone()).unwrap();

        // Release all locks
        let result = manager.release_all();
        assert!(result.is_ok());

        // Try locking again (should succeed because locks were released)
        let result = manager.lock_document("doc1", lock_type);
        assert!(result.is_ok());
    }
}
