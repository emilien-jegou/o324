use clap::Args;
use o324_storage::StorageBox;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_command: Command, storage: &StorageBox) -> eyre::Result<()> {
    storage.debug_message();
    Ok(())
}
