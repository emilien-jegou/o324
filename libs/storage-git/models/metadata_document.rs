use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, Default)]
pub struct MetadataDocument {
    /// Current task id
    pub current: Option<String>,
    /// Ordered list of tasks id
    pub task_refs: BTreeSet<String>,
}
