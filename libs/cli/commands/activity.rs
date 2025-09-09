use crate::utils::command_error;
use crate::utils::time::{calculate_date_range, convert_to_utc_range, DateRange, UtcDateRangeInfo};
use chrono::{DateTime, Duration, Local, NaiveDate, Timelike};
use clap::Args;
use colored::Colorize;
use o324_dbus::dto::ActivityDto;
use o324_dbus::proxy::O324ServiceProxy;
use std::collections::HashMap;

#[derive(Args, Debug)]
pub struct Command {
    #[clap(long, global = true)]
    json: bool,

    /// Show a detailed breakdown of applications within each session.
    #[clap(long, action)]
    detailed: bool,

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

/// Formats a duration, showing only the largest significant unit (e.g., "32m", "10s").
fn format_short_duration(duration: Duration) -> String {
    if duration.num_weeks() > 0 {
        format!("{}w", duration.num_weeks())
    } else if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds().max(1)) // Show at least 1s
    }
}

/// Formats a duration for a time gap, showing more detail (e.g., "2m1s").
fn format_gap_duration(gap: Duration) -> String {
    let d = gap.num_days();
    let h = gap.num_hours() % 24;
    let m = gap.num_minutes() % 60;
    let s = gap.num_seconds() % 60;

    let mut parts = Vec::new();
    if d > 0 {
        parts.push(format!("{}d", d));
    }
    if h > 0 {
        parts.push(format!("{}h", h));
    }
    if m > 0 {
        parts.push(format!("{}m", m));
    }
    if s > 0 {
        parts.push(format!("{}s", s));
    }

    if parts.is_empty() {
        return "0s".to_string();
    }

    parts.into_iter().take(2).collect::<String>()
}

/// Represents a logical user session with a primary application focus.
struct Session<'a> {
    activities: Vec<&'a ActivityDto>,
    primary_app_name: String,
}

impl<'a> Session<'a> {
    fn new(first_activity: &'a ActivityDto) -> Self {
        Self {
            activities: vec![first_activity],
            primary_app_name: first_activity.app_name.clone(),
        }
    }

    fn add_activity(&mut self, activity: &'a ActivityDto) {
        self.activities.push(activity);
    }

    fn start_ts(&self) -> u64 {
        self.activities.first().map_or(0, |a| a.start)
    }

    fn end_ts(&self) -> u64 {
        self.activities.last().map_or(0, |a| a.last_active)
    }
}

/// Groups activities into sessions based on time gaps and application focus.
fn group_activities_into_sessions(activities: &[ActivityDto]) -> Vec<Session> {
    if activities.is_empty() {
        return Vec::new();
    }

    const SESSION_GAP_THRESHOLD_MS: u64 = 5 * 60 * 1000;
    const FOCUS_SHIFT_THRESHOLD_MS: u64 = 90 * 1000;

    let mut sessions: Vec<Session> = Vec::new();
    let mut current_session = Session::new(&activities[0]);

    for i in 1..activities.len() {
        let prev_act = current_session.activities.last().unwrap();
        let current_act = &activities[i];

        let gap_ms = current_act.start.saturating_sub(prev_act.last_active);
        let current_act_duration_ms = current_act.last_active.saturating_sub(current_act.start);

        let mut create_new_session = false;

        if gap_ms > SESSION_GAP_THRESHOLD_MS {
            create_new_session = true;
        } else if current_act.app_name != current_session.primary_app_name {
            if current_act_duration_ms > FOCUS_SHIFT_THRESHOLD_MS {
                create_new_session = true;
            }
        }

        if create_new_session {
            sessions.push(current_session);
            current_session = Session::new(current_act);
        } else {
            current_session.add_activity(current_act);
        }
    }
    sessions.push(current_session);

    sessions
}

/// Prints activities grouped by session, with an option for detailed view.
fn print_activities_grouped(activities: &[ActivityDto], detailed_view: bool) {
    if activities.is_empty() {
        println!("{}", "No activities found.".dimmed());
        return;
    }

    let sessions = group_activities_into_sessions(activities);

    let mut last_date: Option<NaiveDate> = None;
    let mut last_session_end_ts: Option<u64> = None;

    for (i, session) in sessions.iter().enumerate() {
        let session_start_ts = session.start_ts();
        let session_end_ts = session.end_ts();

        let session_start_dt =
            DateTime::from_timestamp((session_start_ts / 1000) as i64, 0).unwrap();
        let session_end_dt = DateTime::from_timestamp((session_end_ts / 1000) as i64, 0).unwrap();
        let date = session_start_dt.date_naive();

        // --- Print Date Header ---
        if Some(date) != last_date {
            if i > 0 {
                println!();
            }
            println!(
                "{}",
                format!("◆ {}", date.format("%Y-%m-%d").to_string())
                    .blue()
                    .bold()
            );
            println!("{}", "│".dimmed());
            last_date = Some(date);
            last_session_end_ts = None;
        }

        if let Some(prev_end_ts) = last_session_end_ts {
            let gap_ms = session_start_ts.saturating_sub(prev_end_ts);
            const PRINT_GAP_THRESHOLD_MS: u64 = 60 * 1000;
            if gap_ms > PRINT_GAP_THRESHOLD_MS {
                let gap_duration = Duration::milliseconds(gap_ms as i64);
                let gap_str = format_gap_duration(gap_duration);
                println!("{}", "│".dimmed());
                println!("{} {} {}", "┊".dimmed(), "⋯".dimmed(), gap_str.dimmed());
                println!("{}", "│".dimmed());
            }
        }

        let mut app_durations: HashMap<String, u64> = HashMap::new();
        let mut total_active_duration_ms: u64 = 0;
        for act in &session.activities {
            let duration_ms = act.last_active.saturating_sub(act.start);
            *app_durations.entry(act.app_name.clone()).or_default() += duration_ms;
            total_active_duration_ms += duration_ms;
        }

        let mut sorted_apps: Vec<_> = app_durations.into_iter().collect();
        sorted_apps.sort_by_key(|&(_, duration)| std::cmp::Reverse(duration));

        let is_last_of_day = match sessions.get(i + 1) {
            None => true,
            Some(next_session) => {
                let next_start_dt =
                    DateTime::from_timestamp((next_session.start_ts() / 1000) as i64, 0).unwrap();
                next_start_dt.date_naive() != date
            }
        };
        let prefix = if is_last_of_day { "╰─" } else { "├─" };

        let start_time = format!(
            "{:02}:{:02}",
            session_start_dt.hour(),
            session_start_dt.minute()
        );
        let end_time = format!(
            "{:02}:{:02}",
            session_end_dt.hour(),
            session_end_dt.minute()
        );
        let total_duration_str = format_short_duration(session_end_dt - session_start_dt);
        let primary_app_str = &session.primary_app_name;

        println!(
            "{} {} → {} - {} ({})",
            prefix.dimmed(),
            start_time.cyan().bold(),
            end_time.cyan(),
            total_duration_str,
            primary_app_str.green()
        );

        // This block is only executed if the --detailed flag is present
        // AND there is more than one app to show.
        if detailed_view && sorted_apps.len() > 1 {
            let breakdown_prefix = if is_last_of_day { "   " } else { "│  " };
            for (j, (app_name, duration_ms)) in sorted_apps.iter().enumerate() {
                let app_duration = Duration::milliseconds(*duration_ms as i64);
                let duration_str = format_gap_duration(app_duration);

                let percent = if total_active_duration_ms > 0 {
                    (duration_ms * 100 / total_active_duration_ms) as u8
                } else {
                    0
                };

                let sub_prefix = if j == sorted_apps.len() - 1 {
                    "╰─"
                } else {
                    "├─"
                };

                println!(
                    "{}{} {} - {} - {}",
                    breakdown_prefix.dimmed(),
                    sub_prefix.dimmed(),
                    format!("{percent}%").yellow(),
                    duration_str.dimmed(),
                    app_name
                );
            }
        }

        last_session_end_ts = Some(session_end_ts);
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
        print_activities_grouped(&activities, command.detailed);
    }

    Ok(())
}
