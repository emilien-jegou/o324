use crate::x11::X11Backend;
use crate::{
    Compositor, DisplayServer, WindowEvent, WindowInfo, WindowProvider, WindowTrackerError,
};
use async_trait::async_trait;
use std::env;
use std::time::Duration;

pub struct GnomeProvider {
    backend: GnomeBackend,
    display_server: DisplayServer,
}

enum GnomeBackend {
    X11(X11Backend),
    Wayland,
}

impl GnomeProvider {
    pub async fn new(display_server: DisplayServer) -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        let backend = match display_server {
            DisplayServer::X11 => GnomeBackend::X11(X11Backend::new()?),
            DisplayServer::Wayland => GnomeBackend::Wayland,
        };
        Ok(Self {
            backend,
            display_server,
        })
    }

    pub async fn detect() -> Option<Compositor> {
        let desktop_session = env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();
        if desktop_session.contains("gnome") {
            if env::var("WAYLAND_DISPLAY").is_ok() {
                Some(Compositor::Gnome(DisplayServer::Wayland))
            } else if env::var("DISPLAY").is_ok() {
                Some(Compositor::Gnome(DisplayServer::X11))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl WindowProvider for GnomeProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        match &self.backend {
            GnomeBackend::X11(backend) => backend.get_active_window_backend().await,
            GnomeBackend::Wayland => Err(WindowTrackerError::NotAvailable(
                "Window tracking on Gnome (Wayland) is not supported due to lack of a stable API."
                    .to_string(),
            )),
        }
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        match &self.backend {
            GnomeBackend::X11(backend) => backend.get_all_windows_backend().await,
            GnomeBackend::Wayland => Err(WindowTrackerError::NotAvailable(
                "Window tracking on Gnome (Wayland) is not supported due to lack of a stable API."
                    .to_string(),
            )),
        }
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        match &self.backend {
            GnomeBackend::X11(backend) => {
                let (tx, rx) = tokio::sync::mpsc::channel(100);
                let provider = backend.clone();
                tokio::spawn(async move {
                    // This is polling-based, same as the generic EWMH provider
                    let mut last_focused: Option<WindowInfo> = None;
                    loop {
                        if let Ok(Some(current)) = provider.get_active_window_backend().await {
                            if last_focused.as_ref().is_none_or(|l| l.id != current.id) {
                                if tx.send(WindowEvent::WindowFocused(current.clone()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                                last_focused = Some(current);
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                });
                Ok(rx)
            }
            GnomeBackend::Wayland => Err(WindowTrackerError::NotAvailable(
                "Window monitoring on Gnome (Wayland) is not supported due to lack of a stable API."
                    .to_string(),
            )),
        }
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Gnome(self.display_server.clone())
    }
}
