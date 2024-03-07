use crate::{managers::git_manager::IGitManager, utils::system_lock::SystemLock};
use o324_storage_core::{LockType, PinFuture, Transaction};
use shaku::Interface;
use std::sync::Arc;

/// A git transaction that should ensure than only one git operation can be started at the same
/// time
pub trait IGitTransaction: Transaction + Interface {}

pub struct GitTransaction {
    git_manager: Arc<dyn IGitManager>,
    lock: SystemLock,
}

const GIT_SYSTEM_LOCK_NAME: &str = "o324-git-transaction";

impl GitTransaction {
    pub fn try_new(
        git_manager: Arc<dyn IGitManager>,
        lock_type: LockType,
    ) -> eyre::Result<GitTransaction> {
        let lock = SystemLock::try_new(GIT_SYSTEM_LOCK_NAME, lock_type)?;

        Ok(Self { git_manager, lock })
    }

    pub fn try_lock(&self) -> eyre::Result<()> {
        self.lock.lock()?;
        Ok(())
    }
}

impl Transaction for GitTransaction {
    fn release(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.git_manager.commit_on_change()?;
            self.lock.unlock()?;
            Ok(())
        })
    }
}

impl IGitTransaction for GitTransaction {}
