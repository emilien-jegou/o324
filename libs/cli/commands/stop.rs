use clap::Args;
use o324_dbus::{proxy::O324ServiceProxy, zbus::Connection};

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command) -> eyre::Result<()> {
    let connection = Connection::session().await?;
    let proxy = O324ServiceProxy::new(&connection).await?;

    let _actions = proxy.stop_current_task().await?;
    Ok(())
}
