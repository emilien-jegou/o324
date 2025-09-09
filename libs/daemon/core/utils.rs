use rand::{distr::Alphanumeric, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_now_ms() -> u64 {
    let now = SystemTime::now();
    let unix_timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    unix_timestamp.as_millis() as u64
}

// New helper function to generate a random alphanumeric ID
pub fn generate_random_id(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
