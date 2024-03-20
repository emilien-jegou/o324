use clap::Args;
use o324_core::Core;
use o324_storage::TaskAction;

#[derive(Args, Debug)]
pub struct Command {
    task_id: String,
}

pub async fn handle(command: Command, core: &Core) -> eyre::Result<()> {
    core.delete_task(command.task_id.clone()).await?;
    crate::dbus::dbus_notify_task_changes(vec![TaskAction::Delete(command.task_id)])?;
    Ok(())
}
