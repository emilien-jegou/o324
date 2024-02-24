use std::{cell::RefCell, rc::Rc};

use super::RebaseIterator;

pub struct Rebase<'repo> {
    pub repository: &'repo git2::Repository,
    pub rebase: Rc<RefCell<git2::Rebase<'repo>>>,
}

impl<'repo> Rebase<'repo> {
    pub fn new(repository: &'repo git2::Repository, rebase: git2::Rebase<'repo>) -> Self {
        Rebase {
            repository,
            rebase: Rc::new(RefCell::new(rebase)),
        }
    }

    // Finalize the rebase process
    pub fn finalize(&mut self) -> Result<(), git2::Error> {
        self.rebase.borrow_mut().finish(None)?;
        Ok(())
    }

    // Abort the rebase process
    pub fn abort(&mut self) -> Result<(), git2::Error> {
        self.rebase.borrow_mut().abort()?;
        Ok(())
    }

    /// Returns an iterator over the rebase operations.
    pub fn iter(&mut self) -> RebaseIterator<'repo> {
        RebaseIterator {
            repository: self.repository,
            rebase: self.rebase.clone(),
        }
    }
}
