use crate::utils::{command_error, display::LogType, task_log_builder::TaskLogBuilder};
use clap::Args;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let task = proxy.cancel_current_task().await?;

    match task {
        Some(canceled_task) => {
            TaskLogBuilder::new(LogType::Success, "Canceled", &canceled_task)
                .with_time_period()
                .print();
        }
        None => {
            log::info!("No task was running to cancel.");
        }
    }

    Ok(())
}
