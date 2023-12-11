use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use o324_core::Core;
use o324_storage::BuiltinStorageType;

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
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize the storage
    Init(commands::init::Command),
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
    pub async fn execute(self, core: &Core) -> eyre::Result<()> {
        match self {
            Self::Start(o) => commands::start::handle(o, core).await?,
            Self::Init(o) => commands::init::handle(o, core).await?,
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

impl From<BuiltinStorageTypeArgs> for BuiltinStorageType {
    fn from(val: BuiltinStorageTypeArgs) -> Self {
        match val {
            BuiltinStorageTypeArgs::Git => BuiltinStorageType::Git,
            BuiltinStorageTypeArgs::Demo => BuiltinStorageType::InMemory,
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
    fn get_config(&self) -> eyre::Result<String> {
        let config_path = self
            .config
            .clone()
            .unwrap_or("~/.config/3to4/config.toml".to_owned());

        Ok(shellexpand::full(&config_path)?.into_owned())
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
    let config_path = args.get_config()?;

    let core = o324_core::load(args.get_storage_type().into(), &config_path).await?;

    // we only print the warning if the user manually specified
    // the --config option and an error occured when retrieving
    // or parsing the config file content.
    if args.config.is_some() {
        if let Err(err) = core.has_found_config_file() {
            println!(
                        "{}",
                        format!("An error occured when loading the config file, the --config option was ignored. Got error: {err}")
                            .yellow()
                    );
        }
    }

    args.command.execute(&core).await?;
    Ok(())
}
