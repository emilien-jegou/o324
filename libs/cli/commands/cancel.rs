use clap::Args;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    let _actions = proxy.cancel_current_task().await?;
    Ok(())
}
