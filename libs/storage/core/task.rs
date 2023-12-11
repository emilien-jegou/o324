use patronus::patronus;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[patronus("TaskUpdate")]
pub struct Task {
    pub id: String,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub start: u64,
    pub end: Option<u64>,
}
