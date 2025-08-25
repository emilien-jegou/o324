use std::fmt::Display;

// Make sure this path is correct for your project
use crate::utils::display::{LogBuilder, LogType};
use crate::utils::displayable_id::DisplayableId;
use chrono::{DateTime, Duration, Local, Utc};
use clap::Args;
use colored::Colorize;
use o324_dbus::{dto, proxy::O324ServiceProxy};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct StatusOutput<'a> {
    task: &'a dto::TaskDto,
    elapsed_secs: i64,
}

#[derive(Args, Debug)]
pub struct Command {
    /// Show json output
    #[clap(long)]
    json: bool,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    let tasks = proxy.list_last_tasks(1).await?;

    if let Some(task) = tasks.first().filter(|t| t.end.is_none()) {
        let elapsed = Utc::now() - ms_to_datetime(task.start)?;

        if command.json {
            let output = StatusOutput {
                task,
                elapsed_secs: elapsed.num_seconds(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            pretty_print_running_task(task, elapsed)?;
        }
    } else {
        if command.json {
            println!("{{}}");
        } else {
            // Use the LogBuilder for a simple message to ensure consistent spacing.
            LogBuilder::new(LogType::Info, "No task is currently running.").print();
        }
    }

    Ok(())
}

fn pretty_print_running_task(task: &dto::TaskDto, elapsed: Duration) -> eyre::Result<()> {
    let start_time_local = ms_to_datetime(task.start)?.with_timezone(&Local);
    let elapsed_str = format_duration_human(elapsed);
    let display_id = DisplayableId::from(task);

    // Construct the main message string for the builder
    let message = format!(
        "Task '{}' is running (for {})",
        task.task_name.cyan().bold(),
        elapsed_str.bold()
    );

    // Use a Box<dyn Display> to handle the two potential types for project display
    let project_display: Box<dyn Display> = if let Some(p) = &task.project {
        Box::new(p.cyan())
    } else {
        Box::new("<none>".italic())
    };

    let tags_display = if !task.tags.is_empty() {
        Some(task.tags.join(", ").yellow())
    } else {
        None
    };

    let started_str = format!(
        "{} (on {})",
        start_time_local.format("%H:%M:%S"),
        task.computer_name.dimmed()
    );

    // Use the LogBuilder to print the structured output
    LogBuilder::new(LogType::Status, message)
        .with_branch("ID", display_id)
        .with_branch("Started", started_str)
        .with_branch("Project", project_display)
        .with_optional_branch("Tags", tags_display)
        .print();

    Ok(())
}

// --- More Precise and Consistent Duration Formatting ---
// Renamed to avoid confusion and match the one from `stop` command.
fn format_duration_human(duration: Duration) -> String {
    let secs = duration.num_seconds();

    if secs < 60 {
        return format!("{}s", secs);
    }

    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    // Always show seconds for a running task for a "live" feel
    if seconds >= 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

fn ms_to_datetime(ms: u64) -> eyre::Result<DateTime<Utc>> {
    DateTime::from_timestamp_millis(ms as i64)
        .ok_or_else(|| eyre::eyre!("Failed to create DateTime from milliseconds: {}", ms))
}

