use derive_more::{Deref, DerefMut};

use crate::PinFuture;

#[derive(Deref, DerefMut)]
#[deref(forward)]
#[deref_mut(forward)]
pub struct TransactionBox(Box<dyn Transaction>);

impl TransactionBox {
    pub fn new(transaction: Box<dyn Transaction>) -> Self {
        Self(transaction)
    }
}

pub trait Transaction: Send + Sync {
    fn release(&mut self) -> PinFuture<eyre::Result<()>>;
}
