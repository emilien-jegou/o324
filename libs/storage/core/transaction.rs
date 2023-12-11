use derive_more::{Deref, DerefMut};

use crate::PinFuture;

#[derive(Deref, DerefMut)]
#[deref(forward)]
#[deref_mut(forward)]
pub struct TransactionBox(Box<dyn Transaction>);

impl TransactionBox {
    pub fn new(storage: impl Transaction + 'static) -> Self {
        Self(Box::new(storage))
    }
}

pub trait Transaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>>;
}
