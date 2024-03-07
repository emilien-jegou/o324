use clap::Args;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {
    /// Name of the project
    #[clap(short, long)]
    verbose: bool,
}

pub async fn handle(_: Command, _core: &Core) -> eyre::Result<()> {
    unimplemented!();
}
