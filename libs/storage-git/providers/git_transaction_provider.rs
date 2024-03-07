use std::{error::Error, sync::Arc};

use o324_storage_core::{PinFuture, Transaction};
use shaku::{HasComponent, Interface, Module, Provider};

use crate::{managers::git_manager::IGitManager, utils::system_lock::SystemLock};

/// A git transaction that should ensure than only one git operation can be started at the same
/// time
pub trait IGitTransaction: Transaction + Interface {}

pub struct GitTransaction {
    git_manager: Arc<dyn IGitManager>,
    lock: SystemLock,
}

const GIT_SYSTEM_LOCK_NAME: &str = "o324-git-transaction";

impl<M: Module + HasComponent<dyn IGitManager>> Provider<M> for GitTransaction {
    type Interface = dyn IGitTransaction;

    fn provide(module: &M) -> Result<Box<Self::Interface>, Box<dyn Error>> {
        let lock = SystemLock::try_new(GIT_SYSTEM_LOCK_NAME)?;
        lock.lock()?;

        Ok(Box::new(Self {
            git_manager: module.resolve(),
            lock,
        }))
    }
}

impl Transaction for GitTransaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.git_manager.commit_on_change()?;
            self.lock.unlock()?;
            Ok(())
        })
    }
}

impl IGitTransaction for GitTransaction {}
