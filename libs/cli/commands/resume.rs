use clap::Args;
use o324_dbus::{dto, proxy::O324ServiceProxy};

#[derive(Args, Debug)]
pub struct Command {
    /// Resume a task by id
    task_id: Option<String>,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    let task = match command.task_id {
        Some(id) => proxy
            .get_task_by_id(id)
            .await?
            .ok_or_else(|| eyre::eyre!("Couldn't find task with given id"))?,
        None => proxy
            .list_last_tasks(1)
            .await?
            .pop()
            .ok_or_else(|| eyre::eyre!("No task to resume"))?,
    };

    if task.end.is_none() {
        return Err(eyre::eyre!("The task is already running."));
    }

    let _actions = proxy
        .start_new_task(dto::StartTaskInputDto {
            task_name: task.task_name,
            project: task.project,
            tags: task.tags,
        })
        .await?;

    Ok(())
}
