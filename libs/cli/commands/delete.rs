use clap::Args;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {
    task_id: String,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    proxy.delete_task(command.task_id.clone()).await?;
    Ok(())
}
