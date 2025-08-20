use crate::{config::Config, core::Core, dbus, storage::models::MODELS};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, config: &Config) -> eyre::Result<()> {
    let core = Core::try_new(config, &MODELS)?;
    dbus::start_dbus_service(core).await?;
    Ok(())
}
