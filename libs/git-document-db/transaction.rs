use crate::utils::system_lock::{SystemLock, SystemLockType};

pub struct Transaction {
    lock: SystemLock,
}

const GIT_SYSTEM_LOCK_NAME: &str = "o324-git-transaction";

/// The transaction ensure that readers are blocked during write operation
/// NB: document level locking is currently not implemented, the whole storage get locked on
/// write operation, I may revisit this in the future
impl Transaction {
    pub fn try_new(lock_type: SystemLockType) -> eyre::Result<Transaction> {
        let lock = SystemLock::try_new(GIT_SYSTEM_LOCK_NAME, lock_type)?;

        Ok(Self { lock })
    }

    pub fn try_lock(&self) -> eyre::Result<()> {
        self.lock.lock()?;
        Ok(())
    }
    fn release(&self) -> eyre::Result<()> {
        //self.git_service.0.commit_on_change()?;
        self.lock.unlock()?;
        Ok(())
    }
}
