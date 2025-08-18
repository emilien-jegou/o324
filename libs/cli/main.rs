use clap::Parser;

mod commands;
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
    /// Subcommand to execute
    #[command(subcommand)]
    command: commands::Command,
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing::setup()?;

    let args = Args::parse();

    args.command.execute().await?;
    Ok(())
}
