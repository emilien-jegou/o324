use std::sync::Arc;

use o324_storage_core::{PinFuture, Transaction};

use crate::{managers::git_manager::IGitManager, utils::semaphore::Semaphore};

pub struct GitTransaction {
    lock: Semaphore,
    git_manager: Arc<dyn IGitManager>,
}

const GIT_SEMAPHORE_NAME: &str = "o324-git-transaction";

impl GitTransaction {
    pub fn try_new(git_manager: Arc<dyn IGitManager>) -> eyre::Result<Self> {
        let mut lock = Semaphore::try_new(GIT_SEMAPHORE_NAME)?;
        lock.try_acquire()?;
        Ok(GitTransaction { lock, git_manager })
    }
}

impl Transaction for GitTransaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.lock.release()?;
            self.git_manager.commit_on_change()?;
            Ok(())
        })
    }
}

impl Drop for GitTransaction {
    fn drop(&mut self) {
        // TODO: this is unsafe
        self.lock.release().expect("Couldn't release semaphore");
    }
}
