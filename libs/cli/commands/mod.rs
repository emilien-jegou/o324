use clap::Subcommand;
use o324_dbus::{proxy::O324ServiceProxy, zbus::Connection};

use crate::utils::command_error;

pub mod activity;
pub mod cancel;
pub mod db;
pub mod delete;
pub mod edit;
pub mod log;
pub mod new;
pub mod playground;
pub mod resume;
pub mod stats;
pub mod status;
pub mod stop;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new task
    New(new::Command),
    /// Display infos on an ongoing task
    Status(status::Command),
    /// Stop an ongoing task
    Stop(stop::Command),
    /// Stop and remove a currently running task
    Cancel(cancel::Command),
    /// Restart last running task
    Resume(resume::Command),
    /// Show last running tasks
    Log(log::Command),
    /// Display statistics about projects and tasks
    Stats(stats::Command),
    /// Update a task information
    Edit(edit::Command),
    /// Remove a task
    Delete(delete::Command),
    /// Query the database directly; this is mainly use in development
    Db(db::Command),
    /// Shows the user window activity
    Activity(activity::Command),
    /// Only in dev mode
    Playground(playground::Command),
}

pub struct CommandOptions {
    pub dbus_connection_name: String,
    pub dbus_service_path: String,
}

impl Command {
    pub async fn execute(self, options: CommandOptions) -> command_error::Result<()> {
        let connection = Connection::session().await.map_err(|e| {
            eyre::eyre!(
                "Failed to connect to the D-Bus session bus.\n\n\
                o324 uses D-Bus to communicate with its background service.\n\
                Please ensure the D-Bus user session is active. This can sometimes fail in environments without a graphical session, like a bare SSH connection.\n\n\
                Internal error: {e}"
            )
        })?;

        // This function is generic over any type `E` that can be displayed.
        fn formulate_proxy_error<E: std::fmt::Display>(e: E) -> eyre::Report {
            eyre::eyre!(
                "Fail to connect to o324-daemon via D-Bus.\n\n\
        The CLI requires the daemon for all core features, such as: \n\
          - tasks management\n\
          - device synchronization\n\
          - activity monitoring (auto stop tasks on AFK)\n\n\
        Please ensure the daemon is running. e.g. for systemd:\n\
        systemctl status o324.service\n\n\
        Internal error: {e}"
            )
        }

        // Now you can use this function for both calls
        let proxy = O324ServiceProxy::builder(&connection)
            .path(options.dbus_service_path)?
            .destination(options.dbus_connection_name)?
            .build()
            .await
            .map_err(formulate_proxy_error)?;

        // Creating the proxy is not enough to verify that
        // the connection was successful.
        let _ = proxy.ping().await.map_err(formulate_proxy_error)?;

        match self {
            Self::New(o) => new::handle(o, proxy).await?,
            Self::Stop(o) => stop::handle(o, proxy).await?,
            Self::Cancel(o) => cancel::handle(o, proxy).await?,
            Self::Status(o) => status::handle(o, proxy).await?,
            Self::Resume(o) => resume::handle(o, proxy).await?,
            Self::Log(o) => log::handle(o, proxy).await?,
            Self::Stats(o) => stats::handle(o, proxy).await?,
            Self::Edit(o) => edit::handle(o, proxy).await?,
            Self::Delete(o) => delete::handle(o, proxy).await?,
            Self::Db(o) => db::handle(o, proxy).await?,
            Self::Activity(o) => activity::handle(o, proxy).await?,
            Self::Playground(o) => playground::handle(o, proxy).await?,
        };

        Ok(())
    }
}
