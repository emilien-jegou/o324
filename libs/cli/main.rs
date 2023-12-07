#![allow(unused_parens)]

use clap::{Parser, Subcommand};

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

#[derive(Parser, Debug)]
#[command(
    name="3to4",
    version,
    author="Emilien Jegou",
    long_about = Some("A CLI & GUI time tracker, learn more on [[GITHUB_LINK]].")
)]
struct Args {
    /// Path of configuration file
    #[arg(short,long, default_value_t = ("~/.config/3to4/config.toml".to_string()))]
    config: String,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Command,
}

pub fn main() {
    let _ = Args::parse();
}
