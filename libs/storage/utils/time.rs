use chrono::{TimeZone, Utc, LocalResult};

// Crockford's Base32 characters
const CROCKFORD32: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn ulid_to_unix_timestamp(ulid: &str) -> eyre::Result<u64> {
    if ulid.len() != 26 {
        return Err(eyre::eyre!("Malformed ULID"));
    }

    let time_str = &ulid[0..10];
    let mut time: u64 = 0;

    for (index, c) in time_str.chars().rev().enumerate() {
        let encoding_index = CROCKFORD32
            .find(c)
            .ok_or_else(|| eyre::eyre!("invalid character found: {c}"))?;
        time += encoding_index as u64 * 32_u64.pow(index as u32);
    }

    Ok(time / 1000)
}

pub fn ulid_to_utc_datetime(ulid: &str) -> eyre::Result<chrono::DateTime<Utc>> {
    let timestamp = ulid_to_unix_timestamp(ulid)?;

    match Utc.timestamp_opt(timestamp as i64, 0) {
        LocalResult::None => Err(eyre::eyre!("No such local time")),
        LocalResult::Single(t) => Ok(t),
        LocalResult::Ambiguous(t1, t2) => Err(eyre::eyre!(
            "Ambiguous local time, ranging from {:?} to {:?}",
            t1,
            t2
        )),
    }
}
