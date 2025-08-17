use clap::Args;
use o324_dbus::{dto, proxy::O324ServiceProxy, zbus::Connection};

#[derive(Args, Debug)]
pub struct Command {
    /// Id of the task to edit or "current" for editing active task
    task_ref: String,

    /// Name of the task
    #[clap(short, long)]
    name: Option<String>,

    /// Project of the task, an empty string will untie the task project
    #[clap(short, long)]
    project: Option<String>,

    /// List of tags of the task
    #[clap(long, use_value_delimiter = true)]
    tags: Option<Vec<String>>,

    /// Start date of the task as unix timestamp
    #[clap(long, use_value_delimiter = true)]
    start: Option<u64>,

    /// End date of the task as unix timestamp
    #[clap(long, use_value_delimiter = true)]
    end: Option<u64>,
}

impl Command {
    /// None -> don't update the project
    /// Some(None) -> untie the project value from the task
    /// Some(Some(x)) -> set the new value for the task project
    pub fn parse_project_value(&self) -> Option<Option<String>> {
        self.project
            .clone()
            .map(|x| if x.is_empty() { None } else { Some(x) })
    }
}

pub async fn handle(command: Command) -> eyre::Result<()> {
    let connection = Connection::session().await?;
    let proxy = O324ServiceProxy::new(&connection).await?;

    let task_update = dto::TaskUpdateDto {
        task_name: command.name.clone().into(),
        project: command.parse_project_value().into(),
        tags: command.tags.into(),
        start: command.start.into(),
        end: command.end.map(Option::Some).into(),
    };

    let _actions = proxy.edit_task(command.task_ref, task_update).await?;
    Ok(())
}
