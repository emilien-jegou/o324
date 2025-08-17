use clap::Args;
use o324_dbus::{dto, proxy::O324ServiceProxy, zbus::Connection};

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command) -> eyre::Result<()> {
    let connection = Connection::session().await?;
    let proxy = O324ServiceProxy::new(&connection).await?;

    let task = proxy
        .list_last_tasks(1)
        .await?
        .pop()
        .ok_or_else(|| eyre::eyre!("No task to resume"))?;

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
