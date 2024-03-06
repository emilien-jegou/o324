use std::{error::Error, sync::Arc};

use o324_storage_core::{PinFuture, Transaction};
use shaku::{HasComponent, Interface, Module, Provider};

use crate::{managers::git_manager::IGitManager, utils::semaphore::Semaphore};

pub trait IGitTransaction: Transaction + Interface {}

pub struct GitTransaction {
    git_manager: Arc<dyn IGitManager>,
    lock: Semaphore,
}

const GIT_SEMAPHORE_NAME: &str = "o324-git-transaction";

impl<M: Module + HasComponent<dyn IGitManager>> Provider<M> for GitTransaction {
    type Interface = dyn IGitTransaction;

    fn provide(module: &M) -> Result<Box<Self::Interface>, Box<dyn Error>> {
        let mut lock = Semaphore::try_new(GIT_SEMAPHORE_NAME)?;
        lock.try_acquire()?;

        Ok(Box::new(Self {
            git_manager: module.resolve(),
            lock,
        }))
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

impl IGitTransaction for GitTransaction {}

impl Drop for GitTransaction {
    fn drop(&mut self) {
        // TODO: this is unsafe
        self.lock.release().expect("Couldn't release semaphore");
    }
}
