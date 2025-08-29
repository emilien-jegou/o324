use clap::Parser;

use crate::utils::exit_code::ExitCode;

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
}

#[tokio::main]
pub async fn main() -> eyre::Result<ExitCode> {
    color_eyre::install()?;
    utils::log::SimpleLogger::init(log::LevelFilter::Trace)?;

    let args = Args::parse();

    if let Err(error) = args.command.execute().await {
        if let utils::command_error::Error::ExitWithError(_, ref report) = error {
            log::error!("{}", report);
        };
        return Ok(*error.code());
    }

    Ok(ExitCode::Success)
}
