use crate::utils::{
    command_error, display::LogType, task_log_builder::TaskLogBuilder, task_ref::TaskRef,
};
use clap::Args;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {
    task_ref: TaskRef,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let task = command.task_ref.get_task(&proxy).await?;

    match proxy.delete_task(task.id).await? {
        Some(deleted_task) => {
            TaskLogBuilder::new(LogType::Success, "Deleted", &deleted_task)
                .with_time_period()
                .print();
        }
        None => {
            log::info!("Task not found.");
        }
    }
    Ok(())
}
