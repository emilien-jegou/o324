use crate::utils::display::{LogBuilder, LogType};
use crate::utils::displayable_id::DisplayableId;
use clap::Args;
use colored::*;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    let task = proxy.stop_current_task().await?;

    match task {
        Some(stopped_task) => {
            let end_time = match stopped_task.end {
                Some(time) => time,
                None => {
                    return Err(eyre::eyre!(
                        "{} {}",
                        "✗".red().bold(),
                        "Failed to stop task: Service returned an inconsistent task state (no end time).".red()
                    ));
                }
            };

            let task_id = DisplayableId::from(&stopped_task);
            let duration_ms = end_time - stopped_task.start;
            let duration = chrono::Duration::milliseconds(duration_ms as i64);

            let message = format!("Stopped task '{}'", stopped_task.task_name.cyan().bold());

            let tags_display = if !stopped_task.tags.is_empty() {
                Some(stopped_task.tags.join(", ").yellow())
            } else {
                None
            };

            LogBuilder::new(LogType::Stop, message)
                .with_branch("ID", task_id)
                .with_branch("Duration", format_duration(duration))
                .with_optional_branch("Project", stopped_task.project.as_ref().map(|p| p.cyan()))
                .with_optional_branch("Tags", tags_display)
                .print();
        }
        None => {
            LogBuilder::new(LogType::Info, "No task was running.").print();
        }
    }

    Ok(())
}

/// A helper function to format a chrono::Duration into a human-readable string.
fn format_duration(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();

    if secs < 60 {
        // For sub-minute durations, also show milliseconds for precision
        if secs < 10 {
            let millis = duration.num_milliseconds() % 1000;
            return format!("{}.{:03}s", secs, millis);
        }
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
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}
