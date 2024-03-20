use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, core: &Core) -> eyre::Result<()> {
    let actions = core.cancel_current_task().await?;
    crate::dbus::dbus_notify_task_changes(actions)?;
    Ok(())
}
