use clap::Args;
use o324_dbus::{dto, proxy::O324ServiceProxy, zbus::Connection};

#[derive(Args, Debug)]
pub struct Command {
    /// Name of the task
    task_name: String,

    /// Name of the project
    #[clap(short, long)]
    project: Option<String>,

    /// List of tags
    #[clap(long, use_value_delimiter = true)]
    tags: Vec<String>,
}

pub async fn handle(command: Command) -> eyre::Result<()> {
    let connection = Connection::session().await?;
    let proxy = O324ServiceProxy::new(&connection).await?;

    let _actions = proxy
        .start_new_task(dto::StartTaskInputDto {
            task_name: command.task_name,
            project: command.project,
            tags: command.tags,
        })
        .await?;

    Ok(())
}
