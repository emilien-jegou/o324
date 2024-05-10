// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use directories_next::ProjectDirs;
use o324_gui_lib::AppConfigInner;
use std::path::PathBuf;

mod tracing;

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

    // TODO: default should be platform specific
    /// Path of configuration file (default: "~/.config/o324/config.toml")
    #[arg(short, long)]
    config: Option<String>,

    // TODO:
    // Do not load dbus daemon
    //#[arg(long)]
    //no_dbus: bool,
}

impl Args {
    fn get_config(&self) -> eyre::Result<String> {
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
pub async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let config_path = args.get_config()?;

    color_eyre::install()?;
    tracing::setup()?;

    let core = o324_core::load(&config_path, args.profile_name.clone())?;

    o324_gui_lib::run(
        core,
        AppConfigInner {
            profile_name: args.profile_name,
            config_path: config_path.to_string(),
        },
    );
    Ok(())
}
