use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_now() -> u64 {
    let now = SystemTime::now();
    let unix_timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    unix_timestamp.as_secs()
}
