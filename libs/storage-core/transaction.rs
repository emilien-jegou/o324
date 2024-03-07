use std::sync::Arc;

use derive_more::Deref;

use crate::PinFuture;

#[derive(Deref, Clone)]
#[deref(forward)]
#[must_use]
pub struct TransactionBox(Arc<dyn Transaction>);

impl TransactionBox {
    pub fn new(transaction: Arc<dyn Transaction>) -> Self {
        Self(transaction)
    }
}

pub trait Transaction: Send + Sync {
    fn release(&self) -> PinFuture<eyre::Result<()>>;
}
