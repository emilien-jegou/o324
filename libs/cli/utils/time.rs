use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc};
use clap::Args;
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

pub fn parse_date_string(date_str: &str, is_end_of_period: bool) -> eyre::Result<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{date_str}-01"), "%Y-%m-%d") {
        if is_end_of_period {
            let next_month_date = if date.month() == 12 {
                date.with_year(date.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
            } else {
                date.with_month(date.month() + 1).unwrap()
            };
            return Ok(next_month_date - Duration::days(1));
        } else {
            return Ok(date);
        }
    }
    eyre::bail!(
        "Invalid date format '{}'. Use YYYY-MM-DD or YYYY-MM.",
        date_str
    )
}

pub fn parse_day_string(day_str: &str) -> eyre::Result<NaiveDate> {
    let today = Local::now().date_naive();
    match day_str {
        "today" => Ok(today),
        "yesterday" => Ok(today - Duration::days(1)),
        s if s.ends_with("d_ago") => {
            let days_ago_str = s.trim_end_matches("d_ago");
            let days_ago: i64 = days_ago_str.parse()?;
            Ok(today - Duration::days(days_ago))
        }
        s => NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
            eyre::eyre!(
                "Invalid date format '{}': {}. Use YYYY-MM-DD, today, yesterday, or Nd_ago.",
                s,
                e
            )
        }),
    }
}

#[derive(Clone, Args, Debug)]
pub struct DateRange {
    /// Number of last days to look at for stats (used by subcommands, fallback)
    #[clap(long, short, global = true, default_value_t = 30)]
    pub last: u64,

    /// Set a custom start date for the stats period (YYYY-MM-DD or YYYY-MM)
    #[clap(long, requires = "end")]
    pub start: Option<String>,

    /// Set a custom end date for the stats period (YYYY-MM-DD or YYYY-MM)
    #[clap(long, requires = "start")]
    pub end: Option<String>,

    /// Show summary/stats for the current week (Mon-Sun)
    #[clap(long, alias = "week", short = 'w', conflicts_with_all = &["last_week", "day", "this_month", "last_month", "start"])]
    pub this_week: bool,

    /// Show summary/stats for the previous week (Mon-Sun)
    #[clap(long, conflicts_with_all = &["this_week", "day", "this_month", "last_month", "start"])]
    pub last_week: bool,

    /// Show summary/stats for the current month
    #[clap(long, conflicts_with_all = &["this_week", "last_week", "day", "last_month", "start"])]
    pub this_month: bool,

    /// Show summary/stats for the previous month
    #[clap(long, conflicts_with_all = &["this_week", "last_week", "day", "this_month", "start"])]
    pub last_month: bool,

    /// Show summary for a specific day (YYYY-MM-DD, today, yesterday, Nd_ago)
    #[clap(long, short, conflicts_with_all = &["this_week", "last_week", "this_month", "last_month", "start"])]
    pub day: Option<String>,
}

/// A type alias for the intermediate result, making function signatures cleaner.
type DateRangeInfo = (NaiveDate, NaiveDate, String, String);

/// The final, timezone-aware result type.
pub type UtcDateRangeInfo = (DateTime<Utc>, DateTime<Utc>, String, String);

/// Converts a NaiveDate range into a UTC DateTime range.
/// The start time is the beginning of the start day in the local timezone.
/// The end time is the beginning of the *day after* the end day, creating an exclusive range.
pub fn convert_to_utc_range(
    (start_date, end_date, title, context): DateRangeInfo,
) -> UtcDateRangeInfo {
    // These unwraps are safe:
    // - `and_hms_opt(0,0,0)` never fails.
    // - `and_local_timezone` is generally safe for midnight, except for rare DST transitions.
    //   For production code, consider `.single()` or `.latest()` for full robustness.
    let start_utc = start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .with_timezone(&Utc);

    let end_utc = (end_date + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .with_timezone(&Utc);

    (start_utc, end_utc, title, context)
}

pub fn calculate_date_range<T: Into<DateRange>>(
    into_cmd: T,
) -> eyre::Result<Option<DateRangeInfo>> {
    let cmd: DateRange = into_cmd.into();
    let today = Local::now().date_naive();

    let res = if let (Some(start_str), Some(end_str)) = (&cmd.start, &cmd.end) {
        let start = parse_date_string(start_str, false)?;
        let end = parse_date_string(end_str, true)?;
        Some((
            start,
            end,
            "Custom Period".to_string(),
            format!("{start} to {end}"),
        ))
    } else if cmd.this_month {
        let start = today.with_day(1).unwrap();
        // A simpler way to get the end of the month: first day of next month, minus one day.
        let end = if today.month() == 12 {
            today.with_year(today.year() + 1).unwrap().with_month(1)
        } else {
            today.with_month(today.month() + 1)
        }
        .unwrap()
        .with_day(1)
        .unwrap()
            - Duration::days(1);

        Some((
            start,
            end,
            "This Month".to_string(),
            today.format("%B %Y").to_string(),
        ))
    } else if cmd.last_month {
        let first_of_this_month = today.with_day(1).unwrap();
        let end_of_last_month = first_of_this_month - Duration::days(1);
        let start_of_last_month = end_of_last_month.with_day(1).unwrap();
        Some((
            start_of_last_month,
            end_of_last_month,
            "Last Month".to_string(),
            start_of_last_month.format("%B %Y").to_string(),
        ))
    } else if cmd.this_week {
        let days_from_mon = today.weekday().num_days_from_monday() as i64;
        let start = today - Duration::days(days_from_mon);
        let end = start + Duration::days(6);
        Some((
            start,
            end,
            "This Week".to_string(),
            format!("{start} to {end}"),
        ))
    } else if cmd.last_week {
        let start_of_this_week =
            today - Duration::days(today.weekday().num_days_from_monday() as i64);
        let end_of_last_week = start_of_this_week - Duration::days(1);
        let start_of_last_week = end_of_last_week - Duration::days(6);
        Some((
            start_of_last_week,
            end_of_last_week,
            "Last Week".to_string(),
            format!("{start_of_last_week} to {end_of_last_week}"),
        ))
    } else if let Some(day_str) = &cmd.day {
        let date = parse_day_string(day_str)?;
        Some((date, date, format!("Day: {day_str}"), date.to_string()))
    } else {
        None
    };

    Ok(res)
}
