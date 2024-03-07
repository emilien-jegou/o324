use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, _core: &Core) -> eyre::Result<()> {
    unimplemented!();
}
