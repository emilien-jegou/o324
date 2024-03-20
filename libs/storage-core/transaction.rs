use std::sync::Arc;

use derive_more::Deref;

use crate::{storage::StorageClient, PinFuture};

#[derive(Deref, Clone)]
#[deref(forward)]
#[must_use]
pub struct TransactionContainer(Arc<dyn Transaction>);

impl TransactionContainer {
    pub fn new(transaction: Arc<dyn Transaction>) -> Self {
        Self(transaction)
    }
}

pub trait Transaction: StorageClient {
    fn release(&self) -> PinFuture<eyre::Result<()>>;
}
