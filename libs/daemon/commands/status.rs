use crate::{
    config::Config,
    core::supervisor::{
        FailurePolicy, ServiceState, ServiceStatus, SupervisedTaskManager, SupervisorMetadata,
    },
};
use chrono::{Duration, TimeZone, Utc};
use clap::Args;
use colored::*;
use comfy_table::{presets, Cell, CellAlignment, Color, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct Command {
    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupervisorMetadataDisplay {
    pub pid: u32,
    pub started_at: i64,
    pub uptime_seconds: i64,
    pub uptime_formatted: String,
}

impl From<SupervisorMetadata> for SupervisorMetadataDisplay {
    fn from(meta: SupervisorMetadata) -> Self {
        let now = Utc::now();
        let start_time = Utc.timestamp_opt(meta.started_at, 0).unwrap();
        let uptime_duration = now.signed_duration_since(start_time);

        Self {
            pid: meta.pid,
            started_at: meta.started_at,
            uptime_seconds: uptime_duration.num_seconds(),
            uptime_formatted: format_duration(uptime_duration),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatusDisplay {
    pub service_name: String,
    pub state: ServiceState,
    pub retry_count: u32,
    pub last_error: Option<String>,
    pub started_at: Option<i64>,
    pub updated_at: i64,
    pub uptime_seconds: Option<i64>,
    pub last_update_seconds_ago: i64,
    pub time_display: String,
}

impl From<ServiceStatus> for ServiceStatusDisplay {
    fn from(status: ServiceStatus) -> Self {
        let now = Utc::now();

        let uptime_seconds = status.started_at.map(|start_time_ts| {
            let start_time = Utc.timestamp_opt(start_time_ts, 0).unwrap();
            now.signed_duration_since(start_time).num_seconds()
        });

        let last_update_time = Utc.timestamp_opt(status.updated_at, 0).unwrap();
        let last_update_seconds_ago = now.signed_duration_since(last_update_time).num_seconds();

        let time_display = match status.state {
            ServiceState::Running | ServiceState::Starting | ServiceState::Retrying => {
                status.started_at.map_or_else(
                    || "N/A".to_string(),
                    |start_ts| {
                        let duration =
                            now.signed_duration_since(Utc.timestamp_opt(start_ts, 0).unwrap());
                        format!("up {}", format_duration(duration))
                    },
                )
            }
            _ => format!("{last_update_seconds_ago}s ago"),
        };

        Self {
            service_name: status.service_name,
            state: status.state,
            retry_count: status.retry_count,
            last_error: status.last_error,
            started_at: status.started_at,
            updated_at: status.updated_at,
            uptime_seconds,
            last_update_seconds_ago,
            time_display,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusOutput {
    pub supervisor: Option<SupervisorMetadataDisplay>,
    pub services: Vec<ServiceStatusDisplay>,
    pub total_count: usize,
    pub timestamp: i64,
}

// --- Command Handler (Unchanged) ---
pub async fn handle(cmd: Command, _config: Config) -> eyre::Result<()> {
    let manager = SupervisedTaskManager::try_new()?;
    let state = manager.get_state()?;

    let supervisor_display = state.metadata.map(Into::into);
    let services_display: Vec<ServiceStatusDisplay> =
        state.services.into_values().map(Into::into).collect();

    match cmd.format {
        OutputFormat::Table => {
            display_table(&supervisor_display, &services_display);
        }
        OutputFormat::Json => {
            let output = StatusOutput {
                supervisor: supervisor_display,
                total_count: services_display.len(),
                services: services_display,
                timestamp: Utc::now().timestamp(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

// --- Display Logic (FINAL, CORRECTED VERSION) ---
fn display_table(
    supervisor: &Option<SupervisorMetadataDisplay>,
    services: &[ServiceStatusDisplay],
) {
    // The `colored` crate is safe to use here because it's not inside a table cell.
    if let Some(meta) = supervisor {
        println!(
            "{} {} (PID: {}, Uptime: {})",
            "Daemon:".bold(),
            "RUNNING".green().bold(),
            meta.pid,
            meta.uptime_formatted.cyan()
        );
    } else {
        println!("{} {}", "Daemon:".bold(), "NOT RUNNING".red().bold());
    }

    if services.is_empty() {
        if supervisor.is_some() {
            println!("\nNo supervised services are currently running.");
        }
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "SERVICE",
        "STATUS",
        "RETRIES",
        "UPTIME / LAST UPDATE",
        "LAST ERROR",
    ]);

    for status in services {
        // THIS IS THE KEY FIX:
        // Create a plain text Cell, then apply styling using comfy-table's own methods.
        // This ensures the layout engine calculates width based on the plain text, not the colored string.
        let status_cell = {
            let cell = Cell::new(status.state.to_string());
            match status.state {
                ServiceState::Starting => cell.fg(Color::Blue),
                ServiceState::Running => cell.fg(Color::Green),
                ServiceState::Retrying => cell.fg(Color::Yellow),
                ServiceState::Stopped => cell.fg(Color::DarkGrey),
                ServiceState::Failed => cell.fg(Color::Red),
            }
        };

        let error_msg = status.last_error.as_deref().unwrap_or("None");

        table.add_row(vec![
            Cell::new(&status.service_name),
            status_cell, // Use the correctly styled cell
            Cell::new(status.retry_count.to_string()).set_alignment(CellAlignment::Right),
            Cell::new(&status.time_display),
            Cell::new(error_msg),
        ]);
    }

    println!("\n{table}");
}

fn format_duration(duration: Duration) -> String {
    let mut seconds = duration.num_seconds();
    if seconds == 0 {
        return "0s".to_string();
    }

    let days = seconds / (24 * 3600);
    seconds %= 24 * 3600;
    let hours = seconds / 3600;
    seconds %= 3600;
    let minutes = seconds / 60;
    seconds %= 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }

    parts.join(" ")
}
