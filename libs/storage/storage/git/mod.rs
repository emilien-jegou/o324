mod config;
mod storage;
mod transaction;

use crate::Task;

pub use config::GitStorageConfig;
pub use transaction::GitTransaction;
pub use storage::GitStorage;

pub type GitDailyDocument = Vec<Task>;
