use crate::{
    commands::start::print_started_task,
    utils::{
        command_error,
        display::{LogBuilder, LogType},
        displayable_id::DisplayableId,
        task_ref::TaskRef,
    },
};
use clap::Args;
use colored::*;
use o324_dbus::{dto, proxy::O324ServiceProxy};

#[derive(Args, Debug)]
pub struct Command {
    /// Resume a task by id. If not provided, the last task will be resumed.
    task_ref: Option<TaskRef>,

    /// New name for the resumed task
    #[clap(short, long)]
    name: Option<String>,

    /// New project for the resumed task. An empty string ("") will untie the task from any project.
    #[clap(short, long)]
    project: Option<String>,

    /// A new list of tags for the resumed task.
    #[clap(long, use_value_delimiter = true)]
    tags: Option<Vec<String>>,
}

impl Command {
    /// Parses the project argument for use in D-Bus calls.
    /// None -> don't update the project (use the old value)
    /// Some(None) -> untie the project value from the task
    /// Some(Some(x)) -> set a new value for the task project
    pub fn parse_project_value(&self) -> Option<Option<String>> {
        self.project
            .clone()
            .map(|x| if x.is_empty() { None } else { Some(x) })
    }
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let task_to_resume = command
        .task_ref
        .clone()
        .unwrap_or(TaskRef::Last)
        .get_task(&proxy)
        .await?;

    let task_to_resume_id = DisplayableId::from(&task_to_resume);
    LogBuilder::new(
        LogType::Info,
        format!(
            "Resuming task '{}' (ID: {})",
            task_to_resume.task_name.cyan(),
            task_to_resume_id
        ),
    )
    .print();

    // A task is running if its `end` time is None.
    if task_to_resume.end.is_none() {
        // Check if any modifications (name, project, or tags) were provided.
        let has_modifications =
            command.name.is_some() || command.project.is_some() || command.tags.is_some();

        if !has_modifications {
            // Case 1: The task is running and no modifications were requested.
            // Return a colorful, descriptive error.
            return Err(eyre::eyre!(
                "{} {}",
                "✗".red().bold(),
                "Tried to resume a running task but no changes were detected, no operation needed."
                    .red()
            )
            .into());
        }
    }

    let start_task_input = dto::StartTaskInputDto {
        task_name: command
            .name
            .clone()
            .unwrap_or_else(|| task_to_resume.task_name.clone()),
        project: command
            .parse_project_value()
            .unwrap_or_else(|| task_to_resume.project.clone()),
        tags: command
            .tags
            .clone()
            .unwrap_or_else(|| task_to_resume.tags.clone()),
    };

    let task = proxy.start_new_task(start_task_input).await?;

    print_started_task(task);
    Ok(())
}
