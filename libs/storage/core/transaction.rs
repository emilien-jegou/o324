use std::ops::{Deref, DerefMut};

use crate::PinFuture;

pub struct TransactionBox(Box<dyn Transaction>);

impl Deref for TransactionBox {
    type Target = dyn Transaction;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for TransactionBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl TransactionBox {
    pub fn new(storage: impl Transaction + 'static) -> Self {
        Self(Box::new(storage))
    }
}

pub trait Transaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>>;
}
