use crate::{
    app,
    config::{self, Config},
    core::supervisor::{start_with_retry, RetryStrategy},
};
use actix::Actor;
use clap::Args;
use std::time::Duration;
use tokio::task::LocalSet;

#[derive(Args, Debug)]
pub struct Command {}

// This function is async, as called from main.rs
pub async fn handle(_: Command, config: Config) -> eyre::Result<()> {
    let storage = config::create_storage_from_config(&config)?;
    let app = app::build(storage.clone(), config)?;

    LocalSet::new()
        .run_until(async move {
            let dbus_actor = app.dbus_actor.clone();
            let _ = start_with_retry(
                move || dbus_actor.clone().start(),
                RetryStrategy::Flat {
                    max_attempts: Some(10),
                    delay: Duration::from_secs(1),
                },
            )
            .await;

            let activity_actor = app.activity_actor.clone();
            let _ = start_with_retry(
                move || activity_actor.clone().start(),
                RetryStrategy::Exponential {
                    max_attempts: Some(5),
                    initial_delay: Duration::from_secs(2),
                    multiplier: 2.0,
                    max_delay: Some(Duration::from_secs(15)),
                },
            )
            .await;

            tracing::info!("All services spawned. Application is running. Press Ctrl-C to exit.");
            // By waiting for the shutdown signal here, we keep the async block (and thus the
            // entire Actix System) alive until the user requests to shut down.
            wait_for_shutdown_signal().await;
            tracing::info!("Shutdown signal received. Cleaning up services and exiting.");
        })
        .await;

    Ok(())
}

/// Waits for a shutdown signal (Ctrl-C or SIGTERM).
async fn wait_for_shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {}", e);
                // This future will pend forever if we can't install the handler,
                // preventing the application from terminating unexpectedly.
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
