use crate::utils::{
    command_error, date_input::DateInput, display::LogType, task_log_builder::TaskLogBuilder,
    task_ref::TaskRef, time_input::TimeInput,
};
use clap::Args;
use o324_dbus::{dto, proxy::O324ServiceProxy};

#[derive(Args, Debug)]
pub struct Command {
    /// Id of the task to edit or "current" for editing active task
    task_ref: TaskRef,

    /// Name of the task
    #[clap(short, long)]
    name: Option<String>,

    /// Project of the task, an empty string will untie the task project
    #[clap(short, long)]
    project: Option<String>,

    /// List of tags of the task
    #[clap(long, use_value_delimiter = true)]
    tags: Option<Vec<String>>,

    /// Start date of the task.
    #[clap(long)]
    start: Option<DateInput>,

    /// End date of the task (Absolute).
    #[clap(long)]
    end: Option<DateInput>,

    /// Duration of the task (Relative).
    /// *Takes precedence over --end if both are provided.*
    #[clap(long, short = 'D')]
    duration: Option<TimeInput>,
}

impl Command {
    pub fn parse_project_value(&self) -> Option<Option<String>> {
        self.project
            .clone()
            .map(|x| if x.is_empty() { None } else { Some(x) })
    }
}

fn resolve_task_end_update(
    end: Option<DateInput>,
    duration: Option<TimeInput>,
) -> Option<dto::TaskUpdateEndDto> {
    match (end, duration) {
        // Conflict: Duration wins
        (Some(_), Some(dur)) => {
            log::warn!("Both --end and --duration provided; Using --duration.");
            Some(dto::TaskUpdateEndDto::Relative(dur.as_millis()))
        }
        (None, Some(dur)) => Some(dto::TaskUpdateEndDto::Relative(dur.as_millis())),
        (Some(date), None) => Some(dto::TaskUpdateEndDto::Absolute(date.as_u64())),
        (None, None) => None,
    }
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let project_update = command.parse_project_value();
    let start_ts = command.start.as_ref().map(|d| d.as_u64());
    let end_dto_val = resolve_task_end_update(command.end, command.duration);
    let task_update = dto::TaskUpdateDto {
        task_name: command.name.clone().into(),
        project: project_update.into(),
        tags: command.tags.into(),
        start: start_ts.into(),
        end: end_dto_val.map(Option::Some).into(),
    };

    let task_g = command.task_ref.get_task(&proxy).await?;
    let task = proxy.edit_task(task_g.id, task_update).await?;

    TaskLogBuilder::new(LogType::Success, "Edited", &task)
        .with_project()
        .with_tags()
        .with_time_period()
        .print();

    Ok(())
}
