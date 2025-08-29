use crate::utils::{
    command_error,
    display::{LogBuilder, LogType},
};
use clap::Args;
use colored::*;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let task = proxy.cancel_current_task().await?;

    match task {
        Some(canceled_task) => {
            let message = format!("Canceled running task '{}'", canceled_task.task_name.cyan());

            LogBuilder::new(LogType::Success, message)
                .with_branch("ID", canceled_task.id)
                .print();
        }
        None => {
            log::info!("No task was running to cancel.");
        }
    }

    Ok(())
}
