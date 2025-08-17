use chrono::{DateTime, Duration, Local, Utc};
use clap::Args;
use colored::Colorize;
use o324_dbus::{dto, proxy::O324ServiceProxy, zbus::Connection};
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

pub async fn handle(command: Command) -> eyre::Result<()> {
    let connection = Connection::session().await?;
    let proxy = O324ServiceProxy::new(&connection).await?;

    // We only need the very last task to determine the current status.
    let tasks = proxy.list_last_tasks(1).await?;

    // Check if there is a last task and if it is currently running (end is None).
    if let Some(task) = tasks.first().filter(|t| t.end.is_none()) {
        // A task is running.
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
        // No task is running.
        if command.json {
            // Output an empty JSON object to indicate nothing is running.
            println!("{{}}");
        } else {
            println!("{}", "No task is currently running.".yellow());
        }
    }

    Ok(())
}

fn pretty_print_running_task(task: &dto::TaskDto, elapsed: Duration) -> eyre::Result<()> {
    let start_time_local = ms_to_datetime(task.start)?.with_timezone(&Local);
    let elapsed_str = format_duration_pretty(elapsed);

    // Header
    println!(
        "{} {} (for {})",
        "▶".green().bold(),
        "ON TASK".green(),
        elapsed_str.bold().cyan()
    );

    // Details using box-drawing characters for a clean look.
    let prefix = "  ├─".dimmed();

    if let Some(project) = &task.project {
        println!("{} {}: {}", prefix, "Project".bold(), project);
    }
    println!("{} {}:   {}", prefix, "Task".bold(), task.task_name);

    if !task.tags.is_empty() {
        let tags_str = task
            .tags
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{} {}:     {}", prefix, "Tags".bold(), tags_str.dimmed());
    }

    println!(
        "{} {}:  {} (on {})",
        prefix,
        "Started".bold(),
        start_time_local.format("%H:%M"),
        task.computer_name.dimmed()
    );

    println!(
        "{} {}:       {}",
        "  ╰─".dimmed(),
        "ID".bold(),
        task.id.dimmed()
    );

    Ok(())
}

// --- Helper Functions (copied from stats/log for consistency) ---

fn format_duration_pretty(duration: Duration) -> String {
    if duration.is_zero() || duration < Duration::zero() {
        return "0s".to_string();
    }
    let total_seconds = duration.num_seconds();
    if total_seconds < 60 {
        return format!("{}s", total_seconds);
    }
    let total_minutes = duration.num_minutes();
    if total_minutes < 60 {
        return format!("{}m", total_minutes);
    }
    let total_hours = duration.num_hours();
    let minutes = total_minutes % 60;
    if minutes > 0 {
        format!("{}h {}m", total_hours, minutes)
    } else {
        format!("{}h", total_hours)
    }
}

fn ms_to_datetime(ms: u64) -> eyre::Result<DateTime<Utc>> {
    DateTime::from_timestamp_millis(ms as i64)
        .ok_or_else(|| eyre::eyre!("Failed to create DateTime from milliseconds: {}", ms))
}
