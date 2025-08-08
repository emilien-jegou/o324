use clap::Args;
use o324_core::{Core, StartTaskInput};

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

pub async fn handle(command: Command, core: &Core) -> eyre::Result<()> {
    let actions = core
        .start_new_task(StartTaskInput {
            task_name: command.task_name,
            project: command.project,
            tags: command.tags,
            computer_name: core.config.core.computer_name.clone(),
        })
        .await?;

    crate::dbus::dbus_notify_task_changes(actions)?;

    Ok(())
}
