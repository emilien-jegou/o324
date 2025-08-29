use crate::utils::command_error;
use crate::utils::display::{LogBuilder, LogType};
use crate::utils::displayable_id::DisplayableId;
use clap::Args;
use colored::*;
use o324_dbus::{dto, proxy::O324ServiceProxy};
use std::fmt::Display;

#[derive(Args, Debug)]
pub struct Command {
    /// Name of the task
    task_name: String,

    /// Name of the project
    #[clap(short, long)]
    project: Option<String>,

    /// List of tags
    #[clap(long, use_value_delimiter = true)]
    tags: Vec<String>,
}

pub fn print_started_task(task: dto::TaskDto) {
    let task_id = DisplayableId::from(&task);
    let message = format!("Started new task '{}'", task.task_name.cyan().bold());

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

    LogBuilder::new(LogType::Start, message)
        .with_branch("ID", task_id)
        .with_branch("Project", project_display)
        .with_optional_branch("Tags", tags_display)
        .print();
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let task = proxy
        .start_new_task(dto::StartTaskInputDto {
            task_name: command.task_name,
            project: command.project,
            tags: command.tags,
        })
        .await?;
    print_started_task(task);
    Ok(())
}
