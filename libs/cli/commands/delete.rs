use clap::Args;
use o324_dbus::{proxy::O324ServiceProxy, zbus::Connection};

#[derive(Args, Debug)]
pub struct Command {
    task_id: String,
}

pub async fn handle(command: Command) -> eyre::Result<()> {
    let connection = Connection::session().await?;
    let proxy = O324ServiceProxy::new(&connection).await?;

    proxy.delete_task(command.task_id.clone()).await?;
    Ok(())
}
