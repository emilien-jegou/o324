use derive_more::{Deref, DerefMut};
use crate::PinFuture;
use super::{transaction::TransactionBox, task::Task};

#[derive(Deref, DerefMut)]
#[deref(forward)]
#[deref_mut(forward)]
pub struct StorageBox(Box<dyn Storage>);

impl StorageBox {
    pub fn new(storage: impl Storage + 'static) -> Self {
        Self(Box::new(storage))
    }
}

pub trait Storage: Sync {
    fn debug_message(&self);

    fn init(&self) -> PinFuture<eyre::Result<()>>;
    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>>;
    fn has_active_task(&self) -> PinFuture<eyre::Result<bool>>;

    fn start_new_task(&self, task: Task) -> PinFuture<eyre::Result<()>>;

    //let txn = self.storage.try_lock().await?;
    //if self.storage.has_active_task().await? == true {
    //self.storage.add_new_task(Task {
    //txn.commit().await?;
}

