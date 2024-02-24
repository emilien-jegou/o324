use clap::{Parser, Subcommand};
use o324_core::Core;

mod commands {
    pub mod cancel;
    pub mod delete;
    pub mod edit;
    pub mod init;
    pub mod log;
    pub mod restart;
    pub mod start;
    pub mod stats;
    pub mod status;
    pub mod stop;
    pub mod sync;
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize the storage
    Init(commands::init::Command),
    ///  Stop any current task and start a new task and and
    Start(commands::start::Command),
    /// Display infos on an ongoing task
    Status(commands::status::Command),
    /// Stop an ongoing task
    Stop(commands::stop::Command),
    /// Stop and remove a currently running task
    Cancel(commands::cancel::Command),
    /// Restart last running task
    Restart(commands::restart::Command),
    /// Show last running tasks
    Log(commands::log::Command),
    /// Display statistics about projects and tasks
    Stats(commands::stats::Command),
    /// Update a task information
    Edit(commands::edit::Command),
    /// Remove a task
    Delete(commands::delete::Command),
    /// Synchronize with external storage
    Sync(commands::sync::Command),
}

impl Command {
    pub async fn execute(self, core: &Core) -> eyre::Result<()> {
        use commands::*;
        match self {
            Self::Start(o) => start::handle(o, core).await?,
            Self::Stop(o) => stop::handle(o, core).await?,
            Self::Init(o) => init::handle(o, core).await?,
            Self::Cancel(o) => cancel::handle(o, core).await?,
            Self::Status(o) => status::handle(o, core).await?,
            Self::Restart(o) => restart::handle(o, core).await?,
            Self::Log(o) => log::handle(o, core).await?,
            Self::Stats(o) => stats::handle(o, core).await?,
            Self::Edit(o) => edit::handle(o, core).await?,
            Self::Delete(o) => delete::handle(o, core).await?,
            Self::Sync(o) => sync::handle(o, core).await?,
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
    /// Profile to use
    #[arg(long)]
    profile_name: Option<String>,

    /// Path of configuration file (default: "~/.config/o324/config.toml")
    #[arg(short, long)]
    config: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Command,
}

impl Args {
    fn get_config(&self) -> eyre::Result<String> {
        let config_path = self
            .config
            .clone()
            .unwrap_or("~/.config/o324/config.toml".to_owned());

        Ok(shellexpand::full(&config_path)?.into_owned())
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let config_path = args.get_config()?;

    let storage_config = o324_core::load(&config_path, args.profile_name)?;

    args.command.execute(&storage_config).await?;
    Ok(())
}
