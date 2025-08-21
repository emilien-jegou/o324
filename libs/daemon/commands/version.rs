use clap::Args;

use crate::config::Config;

#[derive(Args, Debug)]
pub struct Command {}

// NB: Should be renamed to resume instead of restart
pub async fn handle(_: Command, _core: Config) -> eyre::Result<()> {
    Ok(())
}
