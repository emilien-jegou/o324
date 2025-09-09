use crate::utils::command_error;
use crate::utils::display::LogType;
use crate::utils::task_log_builder::TaskLogBuilder;
use crate::utils::time::ms_to_datetime;
use chrono::Utc;
use clap::Args;
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

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let tasks = proxy.list_last_tasks(0, 1).await?;

    if let Some(task) = tasks.first().filter(|t| t.end.is_none()) {
        let elapsed = Utc::now() - ms_to_datetime(task.start)?;

        if command.json {
            let output = StatusOutput {
                task,
                elapsed_secs: elapsed.num_seconds(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            TaskLogBuilder::new(LogType::Start, "Running", task)
                .with_project()
                .with_tags()
                .with_elapsed(elapsed)
                .print();
        }
    } else if command.json {
        println!("{{}}");
    } else {
        // Use the LogBuilder for a simple message to ensure consistent spacing.
        log::info!("No task is currently running.");
    }

    Ok(())
}
