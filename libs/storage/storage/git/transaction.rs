use crate::{utils::semaphore::Semaphore, PinFuture, Transaction};

pub struct GitTransaction {
    lock: Semaphore,
}

const GIT_SEMAPHORE_NAME: &str = "o324-git-transaction";

impl GitTransaction {
    pub fn try_new() -> eyre::Result<Self> {
        let mut lock = Semaphore::try_new(GIT_SEMAPHORE_NAME)?;
        lock.try_acquire()?;
        Ok(GitTransaction { lock })
    }
}

impl Transaction for GitTransaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.lock.release()?;
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
