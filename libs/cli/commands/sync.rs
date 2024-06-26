use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, core: &Core) -> eyre::Result<()> {
    core.synchronize().await?;
    // NB: We should send a dbus refresh event here
    Ok(())
}
