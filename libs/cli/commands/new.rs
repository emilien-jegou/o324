use crate::utils::{
    command_error, date_input::DateInput, display::LogType, task_log_builder::TaskLogBuilder,
    time_input::TimeInput,
};
use clap::Args;
use o324_dbus::{dto, proxy::O324ServiceProxy};

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

    /// Start date of the task.
    #[clap(long)]
    start: Option<DateInput>,

    /// End date of the task (Absolute).
    #[clap(long, requires = "start")]
    end: Option<DateInput>,

    /// Duration of the task (Relative).
    /// *Takes precedence over --end if both are provided.*
    #[clap(long, short = 'D', requires = "start")]
    duration: Option<TimeInput>,
}

pub fn print_started_task(task: dto::TaskDto) -> eyre::Result<()> {
    TaskLogBuilder::new(LogType::Start, "Started", &task)
        .with_project()
        .with_tags()
        .with_time_period()
        .print();

    Ok(())
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
    let start_ts = command.start.as_ref().map(|d| d.as_u64());
    let end_dto = resolve_task_end_update(command.end, command.duration);

    let task = proxy
        .start_new_task(dto::StartTaskInputDto {
            task_name: command.task_name,
            project: command.project,
            tags: command.tags,
            start: start_ts,
            end: end_dto,
        })
        .await?;
    print_started_task(task)?;
    Ok(())
}
