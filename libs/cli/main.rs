use clap::{Parser, Subcommand};

mod commands {
    pub mod cancel;
    pub mod delete;
    pub mod edit;
    pub mod log;
    pub mod resume;
    pub mod start;
    pub mod stats;
    pub mod status;
    pub mod stop;
}

mod tracing;

#[derive(Subcommand, Debug)]
pub enum Command {
    ///  Stop any current task and start a new task and and
    Start(commands::start::Command),
    /// Display infos on an ongoing task
    Status(commands::status::Command),
    /// Stop an ongoing task
    Stop(commands::stop::Command),
    /// Stop and remove a currently running task
    Cancel(commands::cancel::Command),
    /// Restart last running task
    Resume(commands::resume::Command),
    /// Show last running tasks
    Log(commands::log::Command),
    /// Display statistics about projects and tasks
    Stats(commands::stats::Command),
    /// Update a task information
    Edit(commands::edit::Command),
    /// Remove a task
    Delete(commands::delete::Command),
}

impl Command {
    pub async fn execute(self) -> eyre::Result<()> {
        use commands::*;
        match self {
            Self::Start(o) => start::handle(o).await?,
            Self::Stop(o) => stop::handle(o).await?,
            Self::Cancel(o) => cancel::handle(o).await?,
            Self::Status(o) => status::handle(o).await?,
            Self::Resume(o) => resume::handle(o).await?,
            Self::Log(o) => log::handle(o).await?,
            Self::Stats(o) => stats::handle(o).await?,
            Self::Edit(o) => edit::handle(o).await?,
            Self::Delete(o) => delete::handle(o).await?,
        };

        Ok(())
    }
}

// Note: for uniformity, we dont use clap `default_value` or `default_value_t` options
#[derive(Parser, Debug)]
#[command(
    name="o324",
    version,
    author="Emilien Jegou",
    long_about = Some("A CLI & GUI time tracker, learn more on [[GITHUB_LINK]].")
)]
struct Args {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing::setup()?;

    let args = Args::parse();

    args.command.execute().await?;
    Ok(())
}
