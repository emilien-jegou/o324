use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, core: &Core) -> eyre::Result<()> {
    core.stop_current_task().await?;
    Ok(())
}
