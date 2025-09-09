use crate::utils::command_error;
use crate::utils::time::{calculate_date_range, convert_to_utc_range, DateRange, UtcDateRangeInfo};
use chrono::{DateTime, Duration, Local, Timelike};
use clap::Args;
use colored::Colorize;
use o324_dbus::dto::ActivityDto;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {
    #[clap(long, global = true)]
    json: bool,

    #[clap(flatten)]
    pub date_range: DateRange,
}

pub fn calculate_date_range_with_default(range: DateRange) -> eyre::Result<UtcDateRangeInfo> {
    let range = calculate_date_range(range)?.unwrap_or_else(|| {
        let today = Local::now().date_naive();
        (today, today, "Today".to_string(), today.to_string())
    });

    Ok(convert_to_utc_range(range))
}

// assumes activities sorted by at (ascending)
fn print_activities_colored(activities: &[ActivityDto]) {
    if activities.is_empty() {
        println!("{}", "No activities found.".dimmed());
        return;
    }

    let mut last_date = None;
    let mut last_dt = None;
    for (i, act) in activities.iter().enumerate() {
        let dt = DateTime::from_timestamp((act.at / 1000) as i64, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).expect("invalid timestamp"));
        let date = dt.date_naive();
        if Some(date) != last_date {
            if i > 0 {
                println!();
            }
            println!(
                "{} {}",
                "◆".blue().bold(),
                date.format("%Y-%m-%d").to_string().bold()
            );
            println!("{}", "│".dimmed());
            last_date = Some(date);
        }
        // Show time gap if over 1 minute
        if let Some(prev_dt) = last_dt {
            let gap: Duration = dt - prev_dt;
            if gap > Duration::minutes(1) {
                let d = gap.num_days();
                let h = gap.num_hours() % 24;
                let m = gap.num_minutes() % 60;
                let s = gap.num_seconds() % 60;
                let mut gap_str = String::new();
                if d > 0 {
                    gap_str.push_str(&format!("{}d", d));
                }
                if h > 0 {
                    gap_str.push_str(&format!("{}h", h));
                }
                if m > 0 && d == 0 {
                    gap_str.push_str(&format!("{}m", m));
                }
                if s > 0 && d == 0 && h == 0 {
                    gap_str.push_str(&format!("{}s", s));
                }
                println!("{} {} {}", "┊".dimmed(), "⋯".dimmed(), gap_str.dimmed());
            }
        }
        // Main activity line
        let time = format!("{:02}:{:02}", dt.hour(), dt.minute());
        println!(
            "{} {} - {}",
            "├─".dimmed(),
            time.cyan().bold(),
            act.app_name.bold()
        );
        last_dt = Some(dt);
    }
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let (start_utc, end_utc, _, _) = calculate_date_range_with_default(command.date_range)?;
    let start_timestamp_ms = start_utc.timestamp_millis() as u64;
    let end_timestamp_ms = end_utc.timestamp_millis() as u64;

    let activities = proxy
        .list_activity_range(start_timestamp_ms, end_timestamp_ms)
        .await?;

    if command.json {
        println!("{}", serde_json::to_string_pretty(&activities)?);
    } else {
        print_activities_colored(&activities);
    }

    Ok(())
}
