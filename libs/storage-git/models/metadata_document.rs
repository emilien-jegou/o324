use git_document_db::{prelude::*, Document};
use std::collections::BTreeSet;

#[derive(Debug, Default, Document)]
pub struct MetadataDocument {
    #[document(id)]
    pub id: String,

    /// Current task id
    pub current: Option<String>,

    /// Ordered list of tasks id
    pub task_refs: BTreeSet<String>,
}
