use crate::utils::{
    command_error,
    display::{LogBuilder, LogType},
    task_ref::TaskRef,
    time,
};
use clap::Args;
use colored::*;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {
    task_ref: TaskRef,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let task = command.task_ref.get_task(&proxy).await?;

    match proxy.delete_task(task.id).await? {
        Some(deleted_task) => {
            let message = format!("Deleted task '{}'", deleted_task.task_name.cyan());
            let time_display = time::format_time_period_for_display(task.start, task.end);

            LogBuilder::new(LogType::Success, message)
                .with_branch("ID", deleted_task.id)
                .with_branch("Time", time_display.dimmed())
                .print();
        }
        None => {
            log::info!("Task not found.");
        }
    }
    Ok(())
}
