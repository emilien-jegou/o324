use chrono::{DateTime, Local};
use colored::Colorize;

pub fn format_time_period_for_display(start: u64, end: Option<u64>) -> String {
    // Use from_timestamp_millis to correctly handle millisecond precision
    let start_dt = DateTime::from_timestamp_millis(start as i64)
        .expect("Invalid start timestamp")
        .with_timezone(&Local);

    let end_display = if let Some(end_timestamp) = end {
        // Use from_timestamp_millis here as well
        let end_dt = DateTime::from_timestamp_millis(end_timestamp as i64)
            .expect("Invalid end timestamp")
            .with_timezone(&Local);

        if start_dt.date_naive() == end_dt.date_naive() {
            // Same day, only show time
            end_dt.format("%H:%M").to_string()
        } else {
            // Different day, show full date and time
            end_dt.format("%Y-%m-%d %H:%M").to_string()
        }
    } else {
        "CURRENT".red().bold().to_string()
    };

    format!("{} - {}", start_dt.format("%Y-%m-%d %H:%M"), end_display)
}
