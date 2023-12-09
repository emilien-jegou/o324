use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use o324_storage::{BuiltinStorageType, Storage};

mod commands {
    pub mod cancel;
    pub mod delete;
    pub mod edit;
    pub mod log;
    pub mod restart;
    pub mod start;
    pub mod stats;
    pub mod status;
    pub mod stop;
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start a new task
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
}

impl Command {
    pub async fn execute(self, storage: Box<dyn Storage>) -> eyre::Result<()> {
        match self {
            Self::Start(o) => commands::start::handle(o, storage).await?,
            _ => unimplemented!(),
        };

        Ok(())
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum BuiltinStorageTypeArgs {
    Git,
    Demo,
}

impl Into<BuiltinStorageType> for BuiltinStorageTypeArgs {
    fn into(self) -> BuiltinStorageType {
        match self {
            Self::Git => BuiltinStorageType::Git,
            Self::Demo => BuiltinStorageType::InMemory,
        }
    }
}

// Note: for uniformity, we dont use clap `default_value` or `default_value_t` options
#[derive(Parser, Debug)]
#[command(
    name="3to4",
    version,
    author="Emilien Jegou",
    long_about = Some("A CLI & GUI time tracker, learn more on [[GITHUB_LINK]].")
)]
struct Args {
    /// Storage type to use (default: git)
    #[arg(long)]
    storage_type: Option<BuiltinStorageTypeArgs>,

    /// Path of configuration file (default: "~/.config/3to4/config.toml")
    #[arg(short, long)]
    config: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Command,
}

impl Args {
    fn get_config(&self) -> String {
        self.config
            .clone()
            .unwrap_or("~/.config/3to4/config.toml".to_owned())
    }

    fn get_storage_type(&self) -> BuiltinStorageTypeArgs {
        self.storage_type
            .clone()
            .unwrap_or(BuiltinStorageTypeArgs::Git)
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let config_path = args.get_config();

    let core = o324_core::load(args.get_storage_type().into(), &config_path).await?;

    // we only print the warning if the user manually specified
    // the --config option and an error occured when retrieving
    // or parsing the config file content.
    if args.config.is_some() {
        if let Err(err) = core.found_config_file {
            println!(
                        "{}",
                        format!("An error occured when loading the config file, the --config option was ignored. Got error: {err}")
                            .yellow()
                    );
        }
    }

    args.command.execute(core.storage).await?;
    Ok(())
}
