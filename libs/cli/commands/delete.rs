use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {
    task_id: String,
}

pub async fn handle(command: Command, core: &Core) -> eyre::Result<()> {
    core.delete_task(command.task_id).await?;
    Ok(())
}
