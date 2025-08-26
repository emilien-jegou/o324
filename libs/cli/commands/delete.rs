use crate::utils::display::{LogBuilder, LogType};
use clap::Args;
use colored::*;
use o324_dbus::proxy::O324ServiceProxy;

#[derive(Args, Debug)]
pub struct Command {
    task_id: String,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    let task = proxy.delete_task(command.task_id.clone()).await?;

    match task {
        Some(deleted_task) => {
            let message = format!("Deleted task '{}'", deleted_task.task_name.cyan());

            LogBuilder::new(LogType::Success, message)
                .with_branch("ID", deleted_task.id)
                .print();
        }
        None => {
            LogBuilder::new(LogType::Info, "Task not found.").print();
        }
    }
    Ok(())
}
