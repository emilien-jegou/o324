mod config;
mod storage;
mod transaction;

use std::time::{Duration, UNIX_EPOCH};

use crate::Task;

pub use config::GitStorageConfig;
pub use storage::GitStorage;
pub use transaction::GitTransaction;
use ulid::Ulid;

pub type GitDailyDocument = Vec<Task>;

pub(crate) fn ulid_from_timestamp(unix_timestamp: u64) -> eyre::Result<String> {
    let system_time = UNIX_EPOCH
        .checked_add(Duration::from_secs(unix_timestamp))
        .ok_or_else(|| eyre::eyre!("Couldn't parse timestamp"))?;

    Ok(Ulid::from_datetime(system_time).to_string())
}

// Create a ulid from a timestamp and fill the second part of the timestamp with the $c character
pub(crate) fn ulid_from_timestamp_with_overwrite(
    unix_timestamp: u64,
    c: char,
) -> eyre::Result<String> {
    let mut ulid = ulid_from_timestamp(unix_timestamp)?;
    ulid.replace_range(10..26, &c.to_string().repeat(16));
    Ok(ulid)
}
