use crate::backends::wayland_backend;
use async_trait::async_trait;
use std::thread; // Use a standard OS thread
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};

use crate::{Compositor, WindowEvent, WindowInfo, WindowProvider, WindowTrackerError};

// The command for our actor
enum WaylandCommand {
    GetActiveWindow {
        responder: oneshot::Sender<Result<Option<WindowInfo>, WindowTrackerError>>,
    },
    GetAllWindows {
        responder: oneshot::Sender<Result<Vec<WindowInfo>, WindowTrackerError>>,
    },
}

// The WaylandProvider is now just a handle that sends commands
// to the dedicated Wayland thread. The sender is thread-safe.
#[derive(Clone)]
pub struct WaylandProvider {
    command_sender: mpsc::Sender<WaylandCommand>,
}

impl WaylandProvider {
    pub async fn try_new() -> Result<Self, WindowTrackerError> {
        tracing::warn!("Window detection for Wayland is experimental and relies on the wlr-foreign-toplevel-management protocol.");

        let (command_sender, mut command_receiver) = mpsc::channel(32);

        // Use a channel to get the initialization result back from the new thread
        let (init_sender, init_receiver) = oneshot::channel();

        // Spawn a dedicated OS thread for all Wayland communication.
        thread::spawn(move || {
            // Create a single-threaded tokio runtime for this thread.
            let rt = Builder::new_current_thread().enable_all().build().unwrap();

            // Run the actor loop within this runtime.
            rt.block_on(async move {
                // Initialize the Wayland backend inside the dedicated thread.
                let mut backend = match wayland_backend::WaylandBackend::new() {
                    Ok(b) => {
                        // Signal success
                        let _ = init_sender.send(Ok(()));
                        b
                    }
                    Err(e) => {
                        // Signal failure with the original error
                        let _ = init_sender.send(Err(e));
                        return;
                    }
                };

                // This is the actor's main loop.
                while let Some(command) = command_receiver.recv().await {
                    match command {
                        WaylandCommand::GetActiveWindow { responder } => {
                            let res = backend.get_active_window_backend().await;
                            let _ = responder.send(res);
                        }
                        WaylandCommand::GetAllWindows { responder } => {
                            let res = backend.get_all_windows_backend().await;
                            let _ = responder.send(res);
                        }
                    }
                }
            });
        });

        // Wait for the initialization result.
        match init_receiver.await {
            Ok(Ok(())) => Ok(Self { command_sender }),
            Ok(Err(e)) => Err(e), // Propagate the original error from WaylandBackend::new()
            Err(_) => Err(WindowTrackerError::CommandFailed(
                "Wayland thread panicked during initialization".to_string(),
            )),
        }
    }
}

#[async_trait]
impl WindowProvider for WaylandProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let (responder, receiver) = oneshot::channel();
        let cmd = WaylandCommand::GetActiveWindow { responder };

        self.command_sender.send(cmd).await.map_err(|_| {
            WindowTrackerError::CommandFailed("Wayland actor thread has been closed".to_string())
        })?;

        receiver.await.map_err(|_| {
            WindowTrackerError::CommandFailed(
                "Wayland actor thread closed before sending a response".to_string(),
            )
        })?
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let (responder, receiver) = oneshot::channel();
        let cmd = WaylandCommand::GetAllWindows { responder };

        self.command_sender.send(cmd).await.map_err(|_| {
            WindowTrackerError::CommandFailed("Wayland actor thread has been closed".to_string())
        })?;

        receiver.await.map_err(|_| {
            WindowTrackerError::CommandFailed(
                "Wayland actor thread closed before sending a response".to_string(),
            )
        })?
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = mpsc::channel(100);

        let provider = self.clone();

        tokio::spawn(async move {
            let mut last_active_window: Option<WindowInfo> = None;

            loop {
                match provider.get_active_window().await {
                    Ok(current_win_opt) => {
                        let last_id = last_active_window.as_ref().map(|w| &w.id);
                        let current_id = current_win_opt.as_ref().map(|w| &w.id);

                        if last_id != current_id {
                            if let Some(focused_window) = &current_win_opt {
                                let event = WindowEvent::WindowFocused(focused_window.clone());
                                if tx.send(event).await.is_err() {
                                    break;
                                }
                            }
                        } else if let (Some(last_win), Some(current_win)) =
                            (&last_active_window, &current_win_opt)
                        {
                            if last_win.title != current_win.title {
                                let event = WindowEvent::WindowTitleChanged(current_win.clone());
                                if tx.send(event).await.is_err() {
                                    break;
                                }
                            }
                        }
                        last_active_window = current_win_opt;
                    }
                    Err(e) => {
                        tracing::error!("Error polling for active window: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            tracing::info!("Window monitoring task finished.");
        });

        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Wayland
    }
}
