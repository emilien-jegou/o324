#[cfg(target_os = "linux")]
use super::RebaseOperation;
use std::{cell::RefCell, rc::Rc};

#[cfg(target_os = "linux")]
pub struct RebaseIterator<'repo> {
    // 'a outlives 'repo
    pub repository: &'repo git2::Repository,
    pub rebase: Rc<RefCell<git2::Rebase<'repo>>>,
}

#[cfg(target_os = "linux")]
impl<'repo> Iterator for RebaseIterator<'repo> {
    type Item = Result<RebaseOperation<'repo>, git2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let rebase_clone = self.rebase.clone();
        self.rebase.borrow_mut().next().map(|res| {
            res.map(|r| RebaseOperation::new(self.repository, r, rebase_clone))
                .map_err(Into::into)
        })
    }
}
