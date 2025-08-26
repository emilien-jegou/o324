use crate::{
    config::{self, Config},
    core::supervisor::{FailurePolicy, RetryStrategy, SupervisedTaskManager},
    services,
};
use clap::Args;
use std::{sync::Arc, time::Duration};

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, config: Config) -> eyre::Result<()> {
    // NB: SIGHUP should reload config
    let storage = config::create_storage_from_config(&config)?;
    let app = Arc::new(services::build(storage.clone(), config)?);
    // NB: failure policy won't be called if retries is infinite
    let supervisor = SupervisedTaskManager::try_new()?.try_claim_ownership(FailurePolicy::Panic)?;

    let _dbus_handle = supervisor.spawn_supervised_task(
        "DBusService",
        RetryStrategy::Exponential {
            max_attempts: None,
            initial_delay: Duration::from_secs(2),
            multiplier: 2.0,
            max_delay: Some(Duration::from_secs(15)),
        },
        {
            let app_cloned = app.clone();
            move || {
                let app = app_cloned.clone();
                async move { app.dbus_service.serve().await }
            }
        },
    );

    let _we_handle = supervisor.spawn_supervised_task(
        "WindowEventService",
        RetryStrategy::Exponential {
            max_attempts: None,
            initial_delay: Duration::from_secs(2),
            multiplier: 2.0,
            max_delay: Some(Duration::from_secs(60)),
        },
        {
            let app_cloned = app.clone();
            move || {
                let app = app_cloned.clone();
                async move { app.window_event_service.start().await }
            }
        },
    );

    tracing::info!("All services spawned. Application is running. Press Ctrl-C to exit.");

    // Wait for a shutdown signal
    wait_for_shutdown_signal().await;

    tracing::info!("Shutdown signal received. Cleaning up services and exiting.");

    // When this function returns, the `_dbus_handle` and `_we_handle` variables
    // will be dropped, triggering the cleanup logic in `SupervisedTaskHandle::drop`.
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
