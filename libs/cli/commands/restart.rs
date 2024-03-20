use clap::Args;
use o324_core::{Core, StartTaskInput};

#[derive(Args, Debug)]
pub struct Command {}

// NB: Should be renamed to resume instead of restart
pub async fn handle(_: Command, core: &Core) -> eyre::Result<()> {
    let task = core
        .list_last_tasks(1)
        .await?
        .pop()
        .ok_or_else(|| eyre::eyre!("No task to restart"))?;

    let actions = core.start_new_task(StartTaskInput {
        task_name: task.task_name,
        project: task.project,
        tags: task.tags,
    })
    .await?;

    crate::dbus::dbus_notify_task_changes(actions)?;

    Ok(())
}
