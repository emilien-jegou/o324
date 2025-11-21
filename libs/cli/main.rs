use clap::Parser;

use crate::{commands::CommandOptions, utils::exit_code::ExitCode};

mod commands;
pub mod utils;

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
    command: commands::Command,

    // Dbus connection name (default: org.o324.Service)
    #[arg(long)]
    dbus_connection_name: Option<String>,

    // Dbus service path (default: /org/o324/Service)
    #[arg(long)]
    dbus_service_path: Option<String>,
}

impl Args {
    pub fn get_dbus_service_path(&self) -> String {
        self.dbus_service_path
            .as_deref()
            .unwrap_or("/org/o324/Service")
            .to_string()
    }

    pub fn get_dbus_connection_name(&self) -> String {
        self.dbus_connection_name
            .as_deref()
            .unwrap_or("org.o324.Service")
            .to_string()
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<ExitCode> {
    color_eyre::install()?;
    utils::log::SimpleLogger::init(log::LevelFilter::Trace)?;

    let args = Args::parse();

    let options = CommandOptions {
        dbus_connection_name: args.get_dbus_connection_name(),
        dbus_service_path: args.get_dbus_service_path(),
    };

    if let Err(error) = args.command.execute(options).await {
        if let utils::command_error::Error::ExitWithError(_, ref report) = error {
            log::error!("{}", report);
        };
        return Ok(*error.code());
    }

    Ok(ExitCode::Success)
}
