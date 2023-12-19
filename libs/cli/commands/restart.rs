use clap::Args;
use o324_core::{Core, StartTaskInput};

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, core: &Core) -> eyre::Result<()> {
    let task = core
        .list_last_tasks(1)
        .await?
        .pop()
        .ok_or_else(|| eyre::eyre!("No task to restart"))?;

    core.start_new_task(StartTaskInput {
        task_name: task.task_name,
        project: task.project,
        tags: task.tags,
    })
    .await?;

    Ok(())
}
