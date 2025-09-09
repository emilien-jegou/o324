use crate::backends::x11::X11Backend;
use crate::{Compositor, WindowEvent, WindowInfo, WindowProvider, WindowTrackerError};
use async_trait::async_trait;
use std::time::Duration;

/// A window provider for generic X11 environments using EWMH.
/// This is often a fallback for desktop environments that don't have a specific provider.
pub struct X11Provider {
    backend: X11Backend,
}

impl X11Provider {
    /// Creates a new X11Provider.
    pub fn try_new() -> Result<Self, WindowTrackerError> {
        Ok(Self {
            backend: X11Backend::try_new()?,
        })
    }
}

#[async_trait]
impl WindowProvider for X11Provider {
    /// Gets the currently active (focused) window.
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        self.backend.get_active_window_backend().await
    }

    /// Gets a list of all open windows.
    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        self.backend.get_all_windows_backend().await
    }

    /// Starts monitoring for window events (e.g., focus changes).
    /// Note: This is a polling-based implementation.
    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let provider = self.backend.clone();

        tokio::spawn(async move {
            let mut last_focused: Option<WindowInfo> = None;
            loop {
                // Poll for the active window
                if let Ok(Some(current)) = provider.get_active_window_backend().await {
                    // If the focused window has changed, send an event
                    if last_focused.as_ref().is_none_or(|l| l.id != current.id) {
                        if tx
                            .send(WindowEvent::WindowFocused(current.clone()))
                            .await
                            .is_err()
                        {
                            // The receiver was dropped, so we can stop polling.
                            break;
                        }
                        last_focused = Some(current);
                    }
                }
                // Wait before polling again
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });

        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::X11
    }
}
