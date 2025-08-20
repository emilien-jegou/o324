use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Timelike, Utc, Weekday};
use clap::{Args, Subcommand};
use colored::{ColoredString, Colorize};
use o324_dbus::{dto, proxy::O324ServiceProxy};
use serde::Serialize;
use std::collections::HashMap;

// --- JSON Output Structs ---

#[derive(Serialize, Debug, Clone)]
struct SessionInfo {
    session_number: usize,
    start_time: String,
    end_time: String,
    total_duration_secs: i64,
    active_duration_secs: i64,
    activity_percentage: i64,
}

#[derive(Serialize, Debug)]
struct TodaySummary {
    date: String,
    total_active_duration_secs: i64,
    total_session_duration_secs: i64,
    activity_percentage: i64,
    session_count: usize,
    first_task_start_time: String,
    last_task_end_time: String,
    sessions: Vec<SessionInfo>,
}

#[derive(Serialize, Debug)]
struct CategorySummaryItem {
    name: String,
    duration_secs: i64,
    percentage: f64,
}

#[derive(Serialize, Debug)]
struct YearSummary {
    year: i32,
    total_duration_secs: i64,
    daily_activity: HashMap<String, i64>, // Key: "YYYY-MM-DD", Value: seconds
}

// --- CLI Command Structure ---

#[derive(Args, Debug)]
pub struct Command {
    /// Number of last days to look at for stats (used by subcommands).
    #[clap(long, short, global = true, default_value_t = 30)]
    last: u64,

    /// Output results in JSON format
    #[clap(long, global = true)]
    json: bool,

    #[command(subcommand)]
    subcommand: Option<StatsSubcommand>,
}

#[derive(Subcommand, Debug, Clone, Copy)]
enum StatsSubcommand {
    /// Display statistics by project
    Project,
    /// Display statistics by tag
    Tag,
    /// Display statistics by day of the week
    Week,
    /// Display statistics by hour of the day
    Hour,
    /// Display a yearly activity heatmap
    Year,
}

async fn handle_generic_subcommand(
    subcommand: StatsSubcommand,
    last: u64,
    json: bool,
    proxy: &O324ServiceProxy<'_>,
) -> eyre::Result<()> {
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(last as i64);
    let start_timestamp_ms = start_date.timestamp_millis() as u64;

    let all_tasks = proxy
        .list_last_tasks(10000)
        .await?
        .into_iter()
        .filter(|task| task.start >= start_timestamp_ms)
        .collect::<Vec<dto::TaskDto>>();

    if !json && all_tasks.is_empty() {
        println!("No tasks found in the last {last} days.");
        return Ok(());
    }

    match subcommand {
        StatsSubcommand::Project => handle_project_stats(&all_tasks, last, json).await?,
        StatsSubcommand::Tag => handle_tag_stats(&all_tasks, last, json).await?,
        StatsSubcommand::Week => handle_week_stats(&all_tasks, last, json).await?,
        StatsSubcommand::Hour => handle_hour_stats(&all_tasks, last, json).await?,
        StatsSubcommand::Year => unreachable!(),
    }
    Ok(())
}

// --- Specific Handlers ---

async fn handle_today_summary(json: bool, proxy: &O324ServiceProxy<'_>) -> eyre::Result<()> {
    // 1. Fetch all tasks for today
    let now_local = Local::now();
    let start_of_day_local = now_local.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let start_of_day_utc = start_of_day_local
        .and_local_timezone(Local)
        .unwrap()
        .with_timezone(&Utc);
    let start_timestamp_ms = start_of_day_utc.timestamp_millis() as u64;

    let mut todays_tasks = proxy
        .list_last_tasks(1000)
        .await?
        .into_iter()
        .filter(|task| task.start >= start_timestamp_ms)
        .collect::<Vec<dto::TaskDto>>();

    if todays_tasks.is_empty() {
        if json {
            println!("{}", serde_json::to_string(&serde_json::json!({}))?);
        } else {
            print_header("Summary for Today", &now_local.date_naive());
            println!("No tasks logged yet today.");
        }
        return Ok(());
    }
    todays_tasks.sort_by_key(|t| t.start);

    // 2. Group tasks into sessions
    let session_break_threshold = Duration::minutes(30);
    let mut sessions_of_tasks: Vec<Vec<&dto::TaskDto>> = Vec::new();
    sessions_of_tasks.push(vec![&todays_tasks[0]]);
    for i in 1..todays_tasks.len() {
        let prev_task = &todays_tasks[i - 1];
        let current_task = &todays_tasks[i];
        let prev_end_ms = prev_task.end.unwrap_or(current_task.start);

        if ms_to_datetime(current_task.start)? - ms_to_datetime(prev_end_ms)?
            > session_break_threshold
        {
            sessions_of_tasks.push(vec![current_task]);
        } else {
            sessions_of_tasks.last_mut().unwrap().push(current_task);
        }
    }

    // 3. Process each session to calculate its stats
    let mut processed_sessions: Vec<SessionInfo> = Vec::new();
    for (i, session_tasks) in sessions_of_tasks.iter().enumerate() {
        let session_start_dt =
            ms_to_datetime(session_tasks.first().unwrap().start)?.with_timezone(&Local);
        let last_task = session_tasks.last().unwrap();
        let session_end_dt = last_task
            .end
            .map(|e| ms_to_datetime(e).unwrap())
            .unwrap_or_else(Utc::now)
            .with_timezone(&Local);

        let active_duration = session_tasks.iter().try_fold(Duration::zero(), |acc, t| {
            Ok::<_, eyre::Report>(acc + task_duration(t)?)
        })?;
        let total_duration =
            session_end_dt.with_timezone(&Utc) - session_start_dt.with_timezone(&Utc);

        processed_sessions.push(SessionInfo {
            session_number: i + 1,
            start_time: session_start_dt.format("%H:%M").to_string(),
            end_time: if last_task.end.is_some() {
                session_end_dt.format("%H:%M").to_string()
            } else {
                "CURRENT".red().to_string()
            },
            total_duration_secs: total_duration.num_seconds(),
            active_duration_secs: active_duration.num_seconds(),
            activity_percentage: if !total_duration.is_zero() {
                (active_duration.num_seconds() * 100) / total_duration.num_seconds()
            } else {
                0
            },
        });
    }

    // 4. Calculate overall summary stats from processed sessions
    // FIX: Add explicit type annotation `: Duration` to help .sum()
    let total_active_duration: Duration = processed_sessions
        .iter()
        .map(|s| Duration::seconds(s.active_duration_secs))
        .sum();
    let total_session_duration: Duration = processed_sessions
        .iter()
        .map(|s| Duration::seconds(s.total_duration_secs))
        .sum();
    let overall_activity_percentage = if !total_session_duration.is_zero() {
        (total_active_duration.num_seconds() * 100) / total_session_duration.num_seconds()
    } else {
        0
    };

    // 5. Generate output
    if json {
        let first_task_start_time = ms_to_datetime(todays_tasks.first().unwrap().start)?
            .with_timezone(&Local)
            .format("%H:%M")
            .to_string();
        let last_task = todays_tasks.last().unwrap();
        let last_task_end_time = if let Some(end_ms) = last_task.end {
            ms_to_datetime(end_ms)?
                .with_timezone(&Local)
                .format("%H:%M")
                .to_string()
        } else {
            "CURRENT".to_string()
        };

        let summary = TodaySummary {
            date: now_local.date_naive().to_string(),
            total_active_duration_secs: total_active_duration.num_seconds(),
            total_session_duration_secs: total_session_duration.num_seconds(),
            activity_percentage: overall_activity_percentage,
            session_count: processed_sessions.len(),
            first_task_start_time,
            last_task_end_time,
            sessions: processed_sessions,
        };
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_header("Summary for Today", &now_local.date_naive());
        println!(
            "{} in {} {} [{}{}{}]",
            format_duration_pretty(total_active_duration).bold().green(),
            processed_sessions.len().to_string().bold(),
            if processed_sessions.len() == 1 {
                "session"
            } else {
                "sessions"
            },
            " ".normal(),
            format!("{overall_activity_percentage}% active").bold(),
            " ".normal()
        );
        let first_time = ms_to_datetime(todays_tasks.first().unwrap().start)?
            .with_timezone(&Local)
            .format("%H:%M");
        let last_task = todays_tasks.last().unwrap();
        let last_time = if let Some(end_ms) = last_task.end {
            ms_to_datetime(end_ms)?
                .with_timezone(&Local)
                .format("%H:%M")
                .to_string()
                .cyan()
        } else {
            "CURRENT".red().bold()
        };
        println!(
            "{}{} {} {} {}",
            "Day started at ".dimmed(),
            // FIX: Convert chrono's formatter to a String before coloring
            first_time.to_string().cyan(),
            "and ended at".dimmed(),
            last_time,
            "\n".normal()
        );
        print_sessions_table(&processed_sessions);
    }
    Ok(())
}

async fn handle_project_stats(tasks: &[dto::TaskDto], days: u64, json: bool) -> eyre::Result<()> {
    let mut summary: HashMap<String, Duration> = HashMap::new();
    let mut total_duration = Duration::zero();
    for task in tasks {
        let duration = task_duration(task)?;
        total_duration += duration;
        if let Some(project) = &task.project {
            *summary.entry(project.clone()).or_default() += duration;
        }
    }
    if json {
        let items = create_category_summary(&summary, total_duration);
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        print_header("Project Breakdown", &days);
        print_summary_table("Project", &summary, total_duration);
    }
    Ok(())
}

async fn handle_tag_stats(tasks: &[dto::TaskDto], days: u64, json: bool) -> eyre::Result<()> {
    let mut summary: HashMap<String, Duration> = HashMap::new();
    let mut total_duration = Duration::zero();
    for task in tasks {
        let duration = task_duration(task)?;
        total_duration += duration;
        for tag in &task.tags {
            *summary.entry(tag.clone()).or_default() += duration;
        }
    }
    if json {
        let items = create_category_summary(&summary, total_duration);
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        print_header("Tag Breakdown", &days);
        print_summary_table("Tag", &summary, total_duration);
    }
    Ok(())
}

async fn handle_week_stats(tasks: &[dto::TaskDto], days: u64, json: bool) -> eyre::Result<()> {
    let mut summary: HashMap<Weekday, Duration> = HashMap::new();
    let mut total_duration = Duration::zero();
    for task in tasks {
        let duration = task_duration(task)?;
        total_duration += duration;
        let start_local = ms_to_datetime(task.start)?.with_timezone(&Local);
        *summary.entry(start_local.weekday()).or_default() += duration;
    }

    let ordered_days = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];
    let data: Vec<(String, Duration)> = ordered_days
        .iter()
        .map(|day| {
            (
                day.to_string(),
                summary.get(day).cloned().unwrap_or_default(),
            )
        })
        .collect();

    if json {
        let items = data
            .iter()
            .map(|(name, duration)| {
                let percentage = if !total_duration.is_zero() {
                    (duration.num_seconds() as f64 / total_duration.num_seconds() as f64) * 100.0
                } else {
                    0.0
                };
                CategorySummaryItem {
                    name: name.clone(),
                    duration_secs: duration.num_seconds(),
                    percentage,
                }
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        print_header("Activity by Day of Week", &days);
        print_temporal_summary("Day", &data, total_duration);
    }
    Ok(())
}

async fn handle_hour_stats(tasks: &[dto::TaskDto], days: u64, json: bool) -> eyre::Result<()> {
    let mut summary: HashMap<u32, Duration> = HashMap::new();
    let mut total_duration = Duration::zero();
    for task in tasks {
        let duration = task_duration(task)?;
        total_duration += duration;
        let start_local = ms_to_datetime(task.start)?.with_timezone(&Local);
        *summary.entry(start_local.hour()).or_default() += duration;
    }

    let mut data: Vec<(String, Duration)> = (0..24)
        .map(|hour| {
            (
                format!("{hour:02}:00"),
                summary.get(&hour).cloned().unwrap_or_default(),
            )
        })
        .collect();

    if json {
        let items = data
            .iter()
            .map(|(name, duration)| {
                let percentage = if !total_duration.is_zero() {
                    (duration.num_seconds() as f64 / total_duration.num_seconds() as f64) * 100.0
                } else {
                    0.0
                };
                CategorySummaryItem {
                    name: name.clone(),
                    duration_secs: duration.num_seconds(),
                    percentage,
                }
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        data.sort_by(|a, b| b.1.cmp(&a.1));
        print_header("Activity by Hour of Day", &days);
        print_temporal_summary("Hour", &data, total_duration);
    }
    Ok(())
}

async fn handle_year_stats<'a>(json: bool, proxy: &O324ServiceProxy<'a>) -> eyre::Result<()> {
    let now = Local::now();
    let year = now.year();
    let start_of_year = NaiveDate::from_ymd_opt(year, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let start_of_year_utc = start_of_year
        .and_local_timezone(Local)
        .unwrap()
        .with_timezone(&Utc);
    let start_timestamp_ms = start_of_year_utc.timestamp_millis() as u64;

    let year_tasks = proxy
        .list_last_tasks(50000)
        .await?
        .into_iter()
        .filter(|task| task.start >= start_timestamp_ms)
        .collect::<Vec<dto::TaskDto>>();

    if !json && year_tasks.is_empty() {
        print_header("Yearly Activity", &year);
        println!("No tasks logged yet this year.");
        return Ok(());
    }

    let mut daily_summary: HashMap<NaiveDate, Duration> = HashMap::new();
    let mut total_year_duration = Duration::zero();
    for task in &year_tasks {
        let duration = task_duration(task)?;
        total_year_duration += duration;
        let date = ms_to_datetime(task.start)?
            .with_timezone(&Local)
            .date_naive();
        *daily_summary.entry(date).or_default() += duration;
    }

    if json {
        let summary = YearSummary {
            year,
            total_duration_secs: total_year_duration.num_seconds(),
            daily_activity: daily_summary
                .iter()
                .map(|(date, duration)| {
                    (date.format("%Y-%m-%d").to_string(), duration.num_seconds())
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_header("Yearly Activity", &year);
        print_year_heatmap(year, &daily_summary);
        println!(
            "\nTotal time tracked this year: {}",
            format_duration_pretty(total_year_duration).bold().green()
        );
    }
    Ok(())
}

// --- Presentation and Helper Functions ---

fn create_category_summary(
    summary: &HashMap<String, Duration>,
    total: Duration,
) -> Vec<CategorySummaryItem> {
    let mut items: Vec<_> = summary
        .iter()
        .map(|(name, duration)| {
            let percentage = if !total.is_zero() {
                (duration.num_seconds() as f64 / total.num_seconds() as f64) * 100.0
            } else {
                0.0
            };
            CategorySummaryItem {
                name: name.clone(),
                duration_secs: duration.num_seconds(),
                percentage,
            }
        })
        .collect();
    items.sort_by(|a, b| b.duration_secs.cmp(&a.duration_secs));
    items
}

fn print_sessions_table(sessions: &[SessionInfo]) {
    println!(
        "{:<10} | {:<15} | {:<10} | {}",
        "Session".underline(),
        "Time Range".underline(),
        "Duration".underline(),
        "Activity".underline()
    );

    for session in sessions {
        println!(
            "{:<10} | {:<15} | {:<10} | {}% active",
            format!("#{}", session.session_number).cyan(),
            format!("{} → {}", session.start_time, session.end_time),
            format_duration_pretty(Duration::seconds(session.total_duration_secs)),
            session.activity_percentage
        );
    }
}

/// Returns a colored cell string based on absolute duration thresholds.
fn get_heatmap_cell(duration: Duration) -> ColoredString {
    const CELL_CHAR: &str = "■";
    let one_min = Duration::minutes(1);
    let one_hr = Duration::hours(1);
    let two_half_hr = Duration::minutes(150);
    let four_hr = Duration::hours(4);
    let six_hr = Duration::hours(6);
    let eight_hr = Duration::hours(8);
    let ten_hr = Duration::hours(10);

    if duration > ten_hr {
        CELL_CHAR.truecolor(177, 148, 255)
    } else if duration > eight_hr {
        CELL_CHAR.truecolor(33, 110, 57)
    } else if duration > six_hr {
        CELL_CHAR.truecolor(48, 161, 78)
    } else if duration > four_hr {
        CELL_CHAR.truecolor(64, 196, 99)
    } else if duration > two_half_hr {
        CELL_CHAR.truecolor(155, 233, 168)
    } else if duration > one_hr {
        CELL_CHAR.truecolor(200, 200, 200)
    } else if duration >= one_min {
        CELL_CHAR.truecolor(150, 150, 150)
    } else {
        CELL_CHAR.truecolor(40, 40, 40)
    }
}

/// Renders the year activity grid.
fn print_year_heatmap(year: i32, daily_summary: &HashMap<NaiveDate, Duration>) {
    let today = Local::now().date_naive();
    let first_day_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    let day_of_week_offset = first_day_of_year.weekday().num_days_from_sunday();
    let grid_start_date = first_day_of_year - Duration::days(day_of_week_offset as i64);
    let mut month_headers = HashMap::<i64, u32>::new();
    for month in 1..=12 {
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let week_num = (first_of_month - grid_start_date).num_weeks();
        month_headers.insert(week_num, month);
    }
    print!("    ");
    for week_idx in 0..53 {
        if let Some(month_num) = month_headers.get(&(week_idx as i64)) {
            let month_str = NaiveDate::from_ymd_opt(year, *month_num, 1)
                .unwrap()
                .format("%b")
                .to_string();
            print!("{month_str:<4}");
        } else {
            print!("  ");
        }
    }
    println!("\n");
    for day_idx in [1, 2, 3, 4, 5, 6, 0] {
        let day_label = match day_idx {
            1 => "Mon",
            3 => "Wed",
            5 => "Fri",
            _ => "",
        };
        print!("{}", format!("{day_label:<4}").dimmed());
        for week_idx in 0..53 {
            let cell_date = grid_start_date + Duration::weeks(week_idx) + Duration::days(day_idx);
            if cell_date.year() != year || cell_date > today {
                print!("  ");
                continue;
            }
            let duration = daily_summary.get(&cell_date).cloned().unwrap_or_default();
            print!("{} ", get_heatmap_cell(duration));
        }
        println!();
    }
    println!();
    print!("          Less ");
    print!("{} ", get_heatmap_cell(Duration::zero()));
    print!("{} ", get_heatmap_cell(Duration::minutes(5)));
    print!("{} ", get_heatmap_cell(Duration::hours(2)));
    print!("{} ", get_heatmap_cell(Duration::hours(3)));
    print!("{} ", get_heatmap_cell(Duration::hours(5)));
    print!("{} ", get_heatmap_cell(Duration::hours(7)));
    print!("{} ", get_heatmap_cell(Duration::hours(9)));
    print!("{} ", get_heatmap_cell(Duration::hours(11)));
    println!("More");
}

fn print_header(title: &str, context: &dyn std::fmt::Display) {
    println!(
        "{} {}\n",
        title.bold().underline(),
        format!("({context})").dimmed()
    );
}

fn print_summary_table(category_name: &str, summary: &HashMap<String, Duration>, total: Duration) {
    if summary.is_empty() {
        println!("No data to display for this category.");
        return;
    }
    println!(
        "{:<20} | {:<12} | {}",
        category_name.underline(),
        "Duration".underline(),
        "Percentage".underline()
    );
    let mut sorted_summary: Vec<_> = summary.iter().collect();
    sorted_summary.sort_by(|a, b| b.1.cmp(a.1));
    for (name, duration) in sorted_summary {
        print_bar_row(name, *duration, total);
    }
}

fn print_temporal_summary(category_name: &str, data: &[(String, Duration)], total: Duration) {
    if data.iter().all(|(_, d)| d.is_zero()) {
        println!("No data to display for this category.");
        return;
    }
    println!(
        "{:<20} | {:<12} | {}",
        category_name.underline(),
        "Duration".underline(),
        "Percentage".underline()
    );
    for (label, duration) in data {
        print_bar_row(label, *duration, total);
    }
}

fn print_bar_row(label: &str, duration: Duration, total: Duration) {
    let percentage = if !total.is_zero() {
        (duration.num_seconds() as f64 / total.num_seconds() as f64) * 100.0
    } else {
        0.0
    };
    let bar_width = (percentage / 4.0).round() as usize;
    let bar = "█".repeat(bar_width).green();
    println!(
        "{:<20} | {:<12} | {:.1}% {}",
        label.cyan(),
        format_duration_pretty(duration),
        percentage,
        bar.dimmed()
    );
}

fn task_duration(task: &dto::TaskDto) -> eyre::Result<Duration> {
    let start = ms_to_datetime(task.start)?;
    let end = task
        .end
        .map(|e| ms_to_datetime(e).unwrap())
        .unwrap_or_else(Utc::now);
    let duration = end - start;
    if duration < Duration::zero() {
        Ok(Duration::zero())
    } else {
        Ok(duration)
    }
}

fn format_duration_pretty(duration: Duration) -> String {
    if duration.is_zero() || duration < Duration::zero() {
        return "0s".to_string();
    }
    let total_seconds = duration.num_seconds();
    if total_seconds < 60 {
        return format!("{total_seconds}s");
    }
    let total_minutes = duration.num_minutes();
    if total_minutes < 60 {
        return format!("{total_minutes}m");
    }
    let total_hours = duration.num_hours();
    let minutes = total_minutes % 60;
    if minutes > 0 {
        format!("{total_hours}h {minutes}m")
    } else {
        format!("{total_hours}h")
    }
}

fn ms_to_datetime(ms: u64) -> eyre::Result<DateTime<Utc>> {
    DateTime::from_timestamp_millis(ms as i64)
        .ok_or_else(|| eyre::eyre!("Failed to create DateTime from milliseconds: {}", ms))
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    if let Some(subcommand) = command.subcommand {
        match subcommand {
            StatsSubcommand::Year => handle_year_stats(command.json, &proxy).await?,
            _ => handle_generic_subcommand(subcommand, command.last, command.json, &proxy).await?,
        }
    } else {
        handle_today_summary(command.json, &proxy).await?;
    }
    Ok(())
}
