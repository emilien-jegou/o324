use crate::{
    utils::{advisory_lock::SystemLockType, system_lock::SystemLock},
    StoreError, StoreResult,
};
use dashmap::{
    mapref::{entry::Entry, one::Ref},
    DashMap,
};
use std::sync::Arc;

pub(crate) const GIT_LOCK_NAME: &str = "git-document-db-lock";

#[derive(Clone)]
pub struct LockManager {
    locks: DashMap<String, Arc<SystemLock>>,
}

impl<'a> LockManager {
    pub fn new() -> Self {
        Self {
            locks: DashMap::new(),
        }
    }

    fn try_lock(
        &'a self,
        lock_name: &str,
        lock_type: SystemLockType,
    ) -> StoreResult<Ref<'a, String, Arc<SystemLock>>> {
        let entry = self.locks.entry(lock_name.to_owned());

        let res = match entry {
            Entry::Occupied(e) => {
                let current = e.into_ref();
                // TODO: proper upgrade/downgrade of the system lock
                current.lock(lock_type).map_err(StoreError::lock_error)?;
                Ok(current)
            }
            Entry::Vacant(e) => {
                let lock = SystemLock::try_new(lock_name).map_err(StoreError::lock_error)?;
                lock.lock(lock_type.clone())
                    .map_err(StoreError::lock_error)?;
                Ok(e.insert(Arc::new(lock)))
            }
        }?
        .downgrade();

        Ok(res)
    }

    pub fn try_lock_document(
        &'a self,
        document_name: &str,
        lock_type: SystemLockType,
    ) -> StoreResult<Ref<'a, String, Arc<SystemLock>>> {
        self.try_lock(&format!("{GIT_LOCK_NAME}-{document_name}"), lock_type)
    }

    pub fn try_lock_store(
        &'a self,
        lock_type: SystemLockType,
    ) -> StoreResult<Ref<'a, String, Arc<SystemLock>>> {
        self.try_lock(GIT_LOCK_NAME, lock_type)
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

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}
