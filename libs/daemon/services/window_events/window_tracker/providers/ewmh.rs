use crate::services::window_events::window_tracker::x11::X11Backend;
use crate::services::window_events::window_tracker::{
    Compositor, WindowEvent, WindowInfo, WindowProvider, WindowTrackerError,
};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

pub struct EwmhProvider {
    backend: X11Backend,
}

impl EwmhProvider {
    pub fn new() -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        Ok(Self {
            backend: X11Backend::new()?,
        })
    }

    pub async fn detect() -> Option<Compositor> {
        None // TODO
    }
}

#[async_trait]
impl WindowProvider for EwmhProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        self.backend.get_active_window_backend().await
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        self.backend.get_all_windows_backend().await
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let provider = self.backend.clone();

        tokio::spawn(async move {
            let mut last_focused_window: Option<WindowInfo> = None;
            loop {
                match provider.get_active_window_backend().await {
                    Ok(current_window) => {
                        let mut should_send_event = false;
                        if let Some(current) = &current_window {
                            if let Some(last) = &last_focused_window {
                                if current.id != last.id || current.title != last.title {
                                    should_send_event = true;
                                }
                            } else {
                                should_send_event = true;
                            }
                        }

                        if should_send_event
                            && tx
                                .send(WindowEvent::WindowFocused(
                                    current_window.as_ref().unwrap().clone(),
                                ))
                                .await
                                .is_err()
                        {
                            break;
                        }
                        last_focused_window = current_window;
                    }
                    Err(e) => {
                        tracing::error!("Polling for active X11 window failed: {}", e);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
                sleep(Duration::from_millis(200)).await;
            }
        });
        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Unknown
    }
}
