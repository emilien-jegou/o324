use chrono::{Duration, NaiveDateTime, TimeZone, Utc};
use chrono_humanize::HumanTime;
use clap::Args;
use colored::Colorize;
use o324_core::Core;
use o324_storage::Task;
use prettytable::{format, row, Table};

#[derive(Args, Debug)]
pub struct Command {
    /// show verbose output
    #[clap(short, long)]
    verbose: bool,

    /// show json output (override the verbose option)
    #[clap(long)]
    json: bool,
}

pub async fn handle(command: Command, core: &Core) -> eyre::Result<()> {
    let tasks = core.list_last_tasks(20).await?;

    if command.json {
        json_output(&tasks).await?;
    } else if command.verbose {
        verbose_output(&tasks).await?;
    } else {
        short_output(&tasks).await?;
    }

    Ok(())
}

// Helper function to format the duration into a more readable format
fn format_duration(duration: Duration) -> String {
    let seconds = duration.num_seconds();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d", days)
    } else if hours > 0 {
        format!("{}H", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

pub async fn short_output(tasks: &Vec<Task>) -> eyre::Result<()> {
    // Create the table
    let mut table = Table::new();

    table.set_format(format::FormatBuilder::new().padding(0, 1).build());

    for task in tasks.iter() {
        let start_naive = NaiveDateTime::from_timestamp_opt(task.start as i64, 0)
            .ok_or_else(|| eyre::eyre!("Failed to create NaiveDateTime"))?;
        let start_time = Utc.from_utc_datetime(&start_naive);

        //let end_time = if let Some(end) = task.end {
        let (humanized, duration) = if let Some(end) = task.end {
            let end_naive = NaiveDateTime::from_timestamp_opt(end as i64, 0)
                .ok_or_else(|| eyre::eyre!("Failed to create NaiveDateTime"))?;
            let end_time = Utc.from_utc_datetime(&end_naive);
            let humanized = HumanTime::from(end_time - Utc::now()).to_string();
            let duration = end_time - start_time;
            (humanized, format_duration(duration))
        } else {
            let duration = Utc::now() - start_time;
            ("Ongoing".to_string(), format_duration(duration))
        };

        table.add_row(row![
            match task.end {
                Some(_) => "".to_string(),
                None => "*".red().bold().to_string(),
            },
            duration.bold().cyan(),
            format!("({}) ", humanized).cyan(),
            format!(
                "{}{}  {}",
                match task.project.as_deref() {
                    Some(project) => format!("{} - ", project.bold()),
                    None => "".to_string(),
                },
                task.task_name,
                task.tags
                    .iter()
                    .map(|x| format!("#{}", x).yellow().to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        ]);
    }

    table.printstd();

    Ok(())
}

pub async fn verbose_output(tasks: &Vec<Task>) -> eyre::Result<()> {
    // Create the table
    let mut table = Table::new();

    table.set_format(format::FormatBuilder::new().padding(1, 1).build());

    // Add a row per time
    table.add_row(row![
        "ID".cyan().bold(),
        "Project".cyan().bold(),
        "Activity".cyan().bold(),
        "Start".cyan().bold(),
        "End".cyan().bold(),
        "Tags".cyan().bold()
    ]);

    for task in tasks.iter() {
        // Convert UNIX timestamp to NaiveDateTime
        let start_naive = NaiveDateTime::from_timestamp_opt(task.start as i64, 0)
            .ok_or_else(|| eyre::eyre!("Failed to create NaiveDateTime"))?;
        let start_time = Utc
            .from_utc_datetime(&start_naive)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let end_time = if let Some(end) = task.end {
            let end_naive = NaiveDateTime::from_timestamp_opt(end as i64, 0)
                .ok_or_else(|| eyre::eyre!("Failed to create NaiveDateTime"))?;
            Utc.from_utc_datetime(&end_naive)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        } else {
            "ONGOING".to_string()
        };

        let tags = if !task.tags.is_empty() {
            format!("{:?}", task.tags)
        } else {
            "-".to_string()
        };

        table.add_row(row![
            task.ulid,
            task.project.as_deref().unwrap_or("-"),
            task.task_name,
            start_time,
            end_time,
            tags
        ]);
    }

    table.printstd();

    Ok(())
}

pub async fn json_output(tasks: &Vec<Task>) -> eyre::Result<()> {
    println!("{}", serde_json::to_string(tasks)?);
    Ok(())
}
