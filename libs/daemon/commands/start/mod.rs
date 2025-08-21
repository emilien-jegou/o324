use crate::{config::Config, services};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, config: Config) -> eyre::Result<()> {
    let app = services::build(config)?;
    app.dbus_service.start_dbus_service().await?;
    Ok(())
}
