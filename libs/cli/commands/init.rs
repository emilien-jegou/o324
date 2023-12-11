use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_command: Command, core: &Core) -> eyre::Result<()> {
    core.initialize().await?;
    Ok(())
}
