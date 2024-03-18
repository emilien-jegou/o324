use crate::module::GitService;
use o324_storage_core::{LockType, PinFuture, Transaction};

pub struct GitTransaction {
    git_service: GitService,
    //lock: SystemLock,
}

impl GitTransaction {
    pub fn try_new(git_service: GitService, _lock_type: LockType) -> eyre::Result<GitTransaction> {
        //let lock = SystemLock::try_new(GIT_SYSTEM_LOCK_NAME, lock_type)?;

        Ok(Self { git_service })
    }

    pub fn try_lock(&self) -> eyre::Result<()> {
        //self.lock.lock()?;
        Ok(())
    }
}

impl Transaction for GitTransaction {
    fn release(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.git_service.0.commit_on_change()?;
            //self.lock.unlock()?;
            Ok(())
        })
    }
}
