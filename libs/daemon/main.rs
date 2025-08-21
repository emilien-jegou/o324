use clap::{Parser, Subcommand};
use directories_next::ProjectDirs;
use std::path::PathBuf;
mod config;
mod core;
mod entities;
mod services;

mod commands {
    pub mod start;
    pub mod version;
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a default config file
    Start(commands::start::Command),
    /// Print the daemon version
    Version(commands::version::Command),
}

impl Command {
    pub async fn execute(self, conf: config::Config) -> eyre::Result<()> {
        use commands::*;
        match self {
            Self::Start(o) => start::handle(o, conf).await?,
            Self::Version(o) => version::handle(o, conf).await?,
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

    #[arg(long)]
    no_dbus: bool,
}

impl Args {
    fn get_config_path(&self) -> eyre::Result<String> {
        let config_path = match &self.config {
            Some(x) => Ok(x.clone()),
            None => {
                // Retrieve project directories specifically for your application
                if let Some(proj_dirs) = ProjectDirs::from("", "emje.dev", "o324") {
                    // Get the path to the configuration directory
                    let config_dir = proj_dirs.config_dir();
                    let config_path: PathBuf = config_dir.join("config.toml");

                    config_path
                        .to_str()
                        .map(|t| t.to_owned())
                        .ok_or_else(|| eyre::eyre!("couldn't convert os path to string"))
                } else {
                    Err(eyre::eyre!("Project directories could not be found."))
                }
            }
        }?;

        Ok(shellexpand::full(&config_path)?.into_owned())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let config_path = args.get_config_path()?;

    core::tracing::setup()?;
    color_eyre::install()?;

    let storage_config = config::load(&config_path).map_err(|e| {
        eyre::eyre!(
            "An error occured when trying to open the configuration file '{}': {}",
            config_path,
            e
        )
    })?;

    args.command.execute(storage_config).await?;
    Ok(())
}
