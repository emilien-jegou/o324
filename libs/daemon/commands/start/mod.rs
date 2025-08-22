use crate::{config::Config, services};
use clap::Args;
use std::{future::Future, time::Duration};

#[derive(Args, Debug)]
pub struct Command {}

// Enum to define the retry strategy for starting a service.
enum RetryStrategy {
    /// Retry a fixed number of times.
    Flat(u32),
    /// Do not retry; attempt to start only once.
    NoRetry,
}

impl RetryStrategy {
    /// Gets the total number of attempts for the strategy.
    fn attempts(&self) -> u32 {
        match self {
            RetryStrategy::Flat(n) => *n,
            RetryStrategy::NoRetry => 1,
        }
    }
}

struct RetryDelay(Duration);

fn spawn_service<F, Fut, T, E>(
    service_name: &'static str,
    retry_strategy: RetryStrategy,
    retry_delay: RetryDelay,
    mut op: F,
) -> tokio::task::JoinHandle<()>
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    tokio::spawn(async move {
        let max_attempts = retry_strategy.attempts();
        let mut last_error: Option<E> = None;

        for attempt in 1..=max_attempts {
            match op().await {
                Ok(_) => {
                    tracing::info!("Service '{}' has shut down gracefully.", service_name);
                    return;
                }
                Err(e) => {
                    tracing::error!(
                        "Service '{}' failed (attempt {}/{}): {}",
                        service_name,
                        attempt,
                        max_attempts,
                        e
                    );
                    last_error = Some(e);

                    if attempt < max_attempts {
                        tracing::info!("Retrying in {:?}...", retry_delay.0);
                        tokio::time::sleep(retry_delay.0).await;
                    }
                }
            }
        }

        panic!(
            "Service '{}' failed to start after {} attempts: {}. Aborting.",
            service_name,
            max_attempts,
            last_error.unwrap()
        );
    })
}

pub async fn handle(_: Command, config: Config) -> eyre::Result<()> {
    let app = std::sync::Arc::new(services::build(config)?);

    // --- FIX APPLIED HERE ---
    let app_for_dbus = app.clone();
    spawn_service(
        "DBusService",
        RetryStrategy::Flat(3),
        RetryDelay(Duration::from_secs(5)),
        // The outer `move` closure captures `app_for_dbus`.
        move || {
            let app = app_for_dbus.clone();
            async move { app.dbus_service.serve().await }
        },
    );

    // --- AND FIX APPLIED HERE ---
    let app_for_window = app.clone();
    spawn_service(
        "WindowEventService",
        RetryStrategy::NoRetry,
        RetryDelay(Duration::from_secs(0)),
        move || {
            let app = app_for_window.clone();
            async move { app.window_event_service.start().await }
        },
    );

    tracing::info!("All services spawned. Application is running.");
    std::future::pending::<()>().await;

    Ok(())
}
