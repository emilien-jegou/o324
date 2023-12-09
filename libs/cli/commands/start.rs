use clap::Args;
use o324_storage::Storage;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_command: Command, storage: Box<dyn Storage>) -> eyre::Result<()> {
    storage.debug_message();
    Ok(())
}
