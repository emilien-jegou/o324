use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use log::debug;
use std::str::FromStr;

/// Enum to handle various date input formats from the CLI.
#[derive(Clone, Debug)]
pub enum DateInput {
    /// Standard Unix Timestamp (stored as milliseconds)
    Timestamp(u64),
    /// Parsed DateTime object converted to UTC
    Date(DateTime<Utc>),
}

impl FromStr for DateInput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug!("Attempting to parse date string: '{}'", s);

        // 1. Try: Unix Timestamp (detects Seconds vs Milliseconds)
        if let Ok(ts) = s.parse::<u64>() {
            // Heuristic: Current timestamp in seconds is ~1.7 billion (10 digits).
            // Current timestamp in millis is ~1.7 trillion (13 digits).
            // Threshold: 100 billion (100_000_000_000).

            if ts < 100_000_000_000 {
                // Input is likely Seconds -> Convert to Milliseconds
                let millis = ts * 1000;
                debug!("Matched format: Unix Timestamp in Seconds ({}) -> Normalized to Milliseconds ({})", ts, millis);
                return Ok(DateInput::Timestamp(millis));
            } else {
                // Input is likely Milliseconds -> Keep as is
                debug!("Matched format: Unix Timestamp in Milliseconds ({})", ts);
                return Ok(DateInput::Timestamp(ts));
            }
        }

        // 2. Try: ISO 8601 / RFC 3339 with Timezone (e.g., "2025-11-30T09:45:00+01:00")
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            debug!("Matched format: RFC 3339 ({})", dt);
            return Ok(DateInput::Date(dt.with_timezone(&Utc)));
        }

        // 3. Try: Naive DateTime (No timezone, assume UTC)
        let formats = [
            "%Y-%m-%dT%H:%M:%S%.f", // ISO standard with millis
            "%Y-%m-%dT%H:%M:%S",    // ISO standard
            "%Y-%m-%d %H:%M:%S",    // SQL/Human readable
        ];

        for fmt in formats {
            if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
                let dt = Utc.from_utc_datetime(&naive);
                debug!("Matched format: Naive DateTime '{}' -> UTC {}", fmt, dt);
                return Ok(DateInput::Date(dt));
            }
        }

        // 4. Try: Naive Date only (assume start of day UTC)
        if let Ok(naive_date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
            let dt = Utc.from_utc_datetime(&naive_dt);
            debug!(
                "Matched format: Date Only ({}) -> Start of day UTC {}",
                naive_date, dt
            );
            return Ok(DateInput::Date(dt));
        }

        debug!(
            "Failed to match any supported date format for input: '{}'",
            s
        );

        Err(format!(
            "Could not parse date '{}'. Supported formats:\n \
            - Timestamp Seconds (1732956300)\n \
            - Timestamp Millis (1764299135281)\n \
            - RFC3339 (2025-11-30T09:45:00+01:00)\n \
            - UTC DateTime (2025-11-30T09:45:00)\n \
            - Date Only (2025-11-30)",
            s
        ))
    }
}

impl DateInput {
    /// Converts the stored date/timestamp into a u64 Unix timestamp (milliseconds).
    pub fn as_u64(&self) -> u64 {
        match self {
            DateInput::Timestamp(ts) => *ts,
            DateInput::Date(dt) => {
                let millis = dt.timestamp_millis();

                if millis < 0 {
                    0
                } else {
                    millis as u64
                }
            }
        }
    }
}
