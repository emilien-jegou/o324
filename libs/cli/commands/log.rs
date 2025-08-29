use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use clap::Args;
use color_eyre::owo_colors::OwoColorize;
use colored::{ColoredString, Colorize};
use o324_dbus::{
    dto::{self},
    proxy::O324ServiceProxy,
};
use std::collections::HashMap;

use crate::utils::{command_error, displayable_id::DisplayableId};

/// A wrapper struct for display purposes, bundling a task with its unique ID info.
#[derive(Debug)]
pub struct DisplayTask<'a> {
    task: &'a dto::TaskDto,
    id: DisplayableId,
}

/// A summary of statistics for a single day.
#[derive(Debug)]
pub struct DaySummary {
    pub date: NaiveDate,
    pub total_active_duration: Duration,
    pub total_session_duration: Duration,
    pub session_count: usize,
}

impl DaySummary {
    /// Calculates the combined activity percentage for the day.
    pub fn activity_percentage(&self) -> i64 {
        if self.total_session_duration.num_seconds() > 0 {
            (self.total_active_duration.num_seconds() * 100)
                / self.total_session_duration.num_seconds()
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub enum NestedElem<'a> {
    BreakSeparator(Duration),
    Task(DisplayTask<'a>),
}

#[derive(Debug)]
pub struct Session<'a> {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub total_duration: Duration,
    pub active_duration: Duration,
    pub elements: Vec<NestedElem<'a>>,
}

impl<'a> Session<'a> {
    pub fn activity_percentage(&self) -> i64 {
        if self.total_duration.num_seconds() > 0 {
            (self.active_duration.num_seconds() * 100) / self.total_duration.num_seconds()
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub enum TopLevelElem<'a> {
    DateSeparator(DaySummary),
    BreakSeparator(Duration),
    Session(Session<'a>),
}

#[derive(Args, Debug)]
pub struct Command {
    /// show json output (override the verbose option)
    #[clap(long)]
    json: bool,
}

pub async fn short_output(tasks: &[dto::TaskDto]) -> eyre::Result<()> {
    if tasks.is_empty() {
        println!("No tasks to show.");
        return Ok(());
    }

    let log_structure = build_log_structure(tasks)?;
    if !log_structure.is_empty() {
        print_log_structure(&log_structure)?;
    }

    Ok(())
}

/// Builds the hierarchical log structure from a flat list of tasks.
fn build_log_structure<'a>(tasks: &'a [dto::TaskDto]) -> eyre::Result<Vec<TopLevelElem<'a>>> {
    if tasks.is_empty() {
        return Ok(vec![]);
    }

    let mut sorted_tasks: Vec<&dto::TaskDto> = tasks.iter().collect();
    sorted_tasks.sort_by_key(|t| t.start);

    // 1. Group tasks into sessions based on a time threshold
    let session_break_threshold = Duration::minutes(30);
    let mut sessions_of_tasks: Vec<Vec<&'a dto::TaskDto>> = Vec::new();
    if !sorted_tasks.is_empty() {
        sessions_of_tasks.push(vec![sorted_tasks[0]]);
        for i in 1..sorted_tasks.len() {
            let prev_task = sorted_tasks[i - 1];
            let current_task = sorted_tasks[i];
            let prev_end_ms = prev_task.end.unwrap_or(current_task.start);
            let prev_end_time = ms_to_datetime(prev_end_ms)?;
            let current_start_time = ms_to_datetime(current_task.start)?;

            if current_start_time - prev_end_time > session_break_threshold {
                sessions_of_tasks.push(vec![current_task]);
            } else {
                sessions_of_tasks.last_mut().unwrap().push(current_task);
            }
        }
    }

    // --- STAGE 1: Process all sessions and calculate daily statistics ---
    let mut all_sessions: Vec<Session<'a>> = Vec::new();
    let mut daily_stats: HashMap<NaiveDate, (Duration, Duration, usize)> = HashMap::new();

    for session_tasks in sessions_of_tasks {
        let session_start_utc = ms_to_datetime(session_tasks.first().unwrap().start)?;
        let session_start_local = session_start_utc.with_timezone(&Local);
        let session_date = session_start_local.date_naive();

        let session_end_utc = match session_tasks.last().unwrap().end {
            Some(end_ms) => ms_to_datetime(end_ms)?,
            None => Utc::now(),
        };
        let session_end_local = session_end_utc.with_timezone(&Local);

        let total_duration = session_end_utc - session_start_utc;
        let active_duration: Duration = session_tasks
            .iter()
            .map(|task| {
                let start = ms_to_datetime(task.start).unwrap();
                let end = task
                    .end
                    .map(|e| ms_to_datetime(e).unwrap())
                    .unwrap_or_else(Utc::now);
                end - start
            })
            .sum();

        // Update daily statistics
        let stats =
            daily_stats
                .entry(session_date)
                .or_insert((Duration::zero(), Duration::zero(), 0));
        stats.0 += active_duration;
        stats.1 += total_duration;
        stats.2 += 1;

        let mut elements: Vec<NestedElem<'a>> = Vec::new();
        for (task_idx, &task) in session_tasks.iter().enumerate() {
            let unique_id = DisplayableId::from(task);
            elements.push(NestedElem::Task(DisplayTask {
                task,
                id: unique_id,
            }));

            if task_idx < session_tasks.len() - 1 {
                let end_time = task
                    .end
                    .map(ms_to_datetime)
                    .transpose()?
                    .unwrap_or_else(Utc::now);
                let next_start_time = ms_to_datetime(session_tasks[task_idx + 1].start)?;
                let intra_break_dur = next_start_time - end_time;
                if intra_break_dur >= Duration::seconds(30) {
                    elements.push(NestedElem::BreakSeparator(intra_break_dur));
                }
            }
        }

        all_sessions.push(Session {
            start_time: session_start_local,
            end_time: session_end_local,
            total_duration,
            active_duration,
            elements,
        });
    }

    // --- STAGE 2: Build the final TopLevelElem vector using pre-calculated stats ---
    let mut result: Vec<TopLevelElem<'a>> = Vec::new();
    let mut last_session_end_time: Option<DateTime<Utc>> = None;
    let mut last_date: Option<NaiveDate> = None;

    for session in all_sessions {
        let current_date = session.start_time.date_naive();

        if let Some(last_end) = last_session_end_time {
            let break_dur = session.start_time.with_timezone(&Utc) - last_end;
            if last_date.unwrap() != current_date {
                result.push(TopLevelElem::BreakSeparator(break_dur));
                let (total_active, total_session, session_count) =
                    *daily_stats.get(&current_date).unwrap();
                result.push(TopLevelElem::DateSeparator(DaySummary {
                    date: current_date,
                    total_active_duration: total_active,
                    total_session_duration: total_session,
                    session_count,
                }));
            } else if break_dur >= Duration::minutes(1) {
                result.push(TopLevelElem::BreakSeparator(break_dur));
            }
        } else {
            let (total_active, total_session, session_count) =
                *daily_stats.get(&current_date).unwrap();
            result.push(TopLevelElem::DateSeparator(DaySummary {
                date: current_date,
                total_active_duration: total_active,
                total_session_duration: total_session,
                session_count,
            }));
        }

        last_session_end_time = Some(session.end_time.with_timezone(&Utc));
        last_date = Some(current_date);
        result.push(TopLevelElem::Session(session));
    }

    Ok(result)
}

/// Colors the activity percentage string based on its value using absolute RGB.
fn colorize_percentage(percentage: i64) -> ColoredString {
    let text = format!("{percentage}% active");
    if percentage <= 55 {
        colored::Colorize::truecolor(&*text, 230, 60, 60)
    } else if percentage <= 65 {
        colored::Colorize::truecolor(&*text, 255, 165, 0)
    } else if percentage <= 75 {
        colored::Colorize::truecolor(&*text, 230, 230, 50)
    } else {
        colored::Colorize::truecolor(&*text, 50, 200, 50)
    }
}

/// Prints a log structure to the console with proper formatting.
fn print_log_structure(log_items: &[TopLevelElem]) -> eyre::Result<()> {
    let total_sessions = log_items
        .iter()
        .filter(|item| matches!(item, TopLevelElem::Session(_)))
        .count();
    let mut session_progress_count = 0;
    let mut daily_session_number = 0;

    // --- MODIFICATION: Use an indexed loop to allow peeking at the next item ---
    for (idx, item) in log_items.iter().enumerate() {
        match item {
            TopLevelElem::DateSeparator(summary) => {
                daily_session_number = 0;

                let duration_string = format_duration_pretty(summary.total_session_duration);
                let duration_part = duration_string.bold();

                let sessions_string = format!(
                    "{} {} {}",
                    "in".dimmed(),
                    summary.session_count.bold(),
                    "sessions".dimmed()
                );

                let active_part_string = format!(
                    "{}{}{}",
                    "[".dimmed(),
                    colorize_percentage(summary.activity_percentage()),
                    "]".dimmed()
                );

                println!("{}", "│".dimmed());
                println!(
                    "{}{} - {} {} {}",
                    "◆ ".blue().bold(),
                    summary.date.format("%Y-%m-%d").blue().bold(),
                    duration_part,
                    sessions_string,
                    active_part_string
                );
                println!("{}", "│".dimmed());
            }

            TopLevelElem::BreakSeparator(duration) => {
                println!("{}", "│".dimmed());
                println!(
                    "{} {} {}",
                    "┊".dimmed(),
                    "⋯".dimmed(),
                    format_duration_pretty(*duration).dimmed(),
                );

                // --- MODIFICATION: Add a spacer only if the next item is a Session ---
                if let Some(next_item) = log_items.get(idx + 1) {
                    if matches!(next_item, TopLevelElem::Session(_)) {
                        println!("{}", "│".dimmed());
                    }
                }
            }
            TopLevelElem::Session(session) => {
                session_progress_count += 1;
                daily_session_number += 1;
                let is_last_session_overall = session_progress_count == total_sessions;
                let header_prefix = if is_last_session_overall {
                    "╰➤"
                } else {
                    "├➤"
                };

                let title_string = format!("Session {daily_session_number}");
                let session_title = title_string.dimmed();

                let time_header_string = format!(
                    "{}{} → {} - {}{}",
                    "[".dimmed(),
                    session.start_time.format("%H:%M").cyan(),
                    session.end_time.format("%H:%M").cyan(),
                    format_duration_pretty(session.total_duration).bold(),
                    "]".dimmed()
                );

                let active_header_string = format!(
                    "{}{}{}",
                    "[".dimmed(),
                    colorize_percentage(session.activity_percentage()),
                    "]".dimmed()
                );
                println!(
                    "{} {} {} {}",
                    header_prefix.dimmed(),
                    session_title,
                    time_header_string,
                    active_header_string
                );

                let content_prefix = if is_last_session_overall {
                    "  "
                } else {
                    "│ "
                };

                for (elem_idx, element) in session.elements.iter().enumerate() {
                    match element {
                        NestedElem::BreakSeparator(duration) => {
                            println!(
                                "{} {}  {} {}",
                                content_prefix.dimmed(),
                                "┊".dimmed(),
                                "⋯".dimmed(),
                                format_duration_pretty(*duration).dimmed()
                            );
                        }
                        NestedElem::Task(display_task) => {
                            let is_last_element_in_session = elem_idx == session.elements.len() - 1;

                            let task = display_task.task;
                            let start_time = ms_to_datetime(task.start)?.with_timezone(&Local);
                            let task_start_time_utc = ms_to_datetime(task.start)?;
                            let task_end_time_utc = task
                                .end
                                .map(|e| ms_to_datetime(e).unwrap())
                                .unwrap_or_else(Utc::now);
                            let task_duration = task_end_time_utc - task_start_time_utc;

                            let duration_string = format_duration_pretty(task_duration);
                            let duration_segment = format!(
                                "{}{}{}",
                                "(".dimmed(),
                                duration_string.cyan().bold(),
                                ")".dimmed()
                            );

                            let (status_icon, time_segment) = if let Some(end_ms) = task.end {
                                let end_dt = ms_to_datetime(end_ms)?.with_timezone(&Local);
                                (
                                    "✓".green(),
                                    format!(
                                        "{} → {}",
                                        start_time.format("%H:%M"),
                                        end_dt.format("%H:%M")
                                    )
                                    .dimmed()
                                    .to_string(),
                                )
                            } else {
                                (
                                    "▶".yellow(),
                                    format!(
                                        "{} {} {}",
                                        start_time.format("%H:%M").dimmed(),
                                        "→".dimmed(),
                                        "CURRENT".red().bold()
                                    ),
                                )
                            };
                            let task_connector = if is_last_element_in_session {
                                "╰─"
                            } else {
                                "├─"
                            };
                            println!(
                                "{} {} {} {} - {} - {} {}",
                                content_prefix.dimmed(),
                                task_connector.dimmed(),
                                status_icon,
                                display_task.id,
                                &task.computer_name.dimmed(),
                                time_segment,
                                duration_segment
                            );
                            let desc_line_prefix = if is_last_element_in_session {
                                format!("{}      ", content_prefix.dimmed())
                            } else {
                                format!("{} {}    ", content_prefix.dimmed(), "│".dimmed())
                            };
                            let tags = task
                                .tags
                                .iter()
                                .map(|t| format!("#{t}"))
                                .collect::<Vec<_>>()
                                .join(" ")
                                .dimmed()
                                .to_string();
                            println!(
                                "{}{} {} {}",
                                desc_line_prefix,
                                match task.project.as_deref() {
                                    Some(p) => format!("{} -", p.bold()),
                                    None => "".to_string(),
                                },
                                task.task_name,
                                tags
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn format_duration_pretty(duration: Duration) -> String {
    if duration < Duration::zero() {
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
    if total_hours < 24 {
        let minutes = total_minutes % 60;
        if minutes > 0 {
            return format!("{total_hours}h{minutes}m");
        } else {
            return format!("{total_hours}h");
        }
    }

    let days = total_hours / 24;
    let hours = total_hours % 24;
    if hours > 0 {
        format!("{days}d{hours}h")
    } else {
        format!("{days}d")
    }
}

fn ms_to_datetime(ms: u64) -> eyre::Result<DateTime<Utc>> {
    DateTime::from_timestamp_millis(ms as i64)
        .ok_or_else(|| eyre::eyre!("Failed to create DateTime from milliseconds: {}", ms))
}

pub async fn json_output(tasks: &[dto::TaskDto]) -> eyre::Result<()> {
    println!("{}", serde_json::to_string_pretty(tasks)?);
    Ok(())
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let tasks = proxy.list_last_tasks(0, 50).await?;

    if command.json {
        json_output(&tasks).await?;
    } else {
        short_output(&tasks).await?;
    }

    Ok(())
}
