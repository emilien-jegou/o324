use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use x11rb::rust_connection::{ConnectError, ConnectionError, ReplyError};

pub mod backends;
pub mod providers;
pub mod utils;

use backends::x11::X11InitError;

use crate::providers::{fht::FhtProvider, wayland::WaylandProvider, x11::X11Provider};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum WindowTrackerError {
    #[error("Unsupported display server")]
    UnsupportedDisplayServer,
    #[error("Compositor not supported: {0}")]
    UnsupportedCompositor(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Feature not available: {0}")]
    NotAvailable(String),
    #[error("X11 connection error: {0}")]
    X11Connection(#[from] ConnectError),
    #[error("X11 initialization error: {0}")]
    X11InitError(#[from] X11InitError),
    #[error("X11 reply error: {0}")]
    X11Reply(#[from] ReplyError),
    #[error("X11 general error: {0}")]
    X11Error(#[from] ConnectionError),
    #[error("DBus error: {0}")]
    DBusError(#[from] zbus::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Wayland connection error: {0}")]
    WaylandConnection(String),
    #[error("Wayland protocol not supported: {0}")]
    WaylandProtocolMissing(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub app_name: String,
    pub pid: Option<u32>,
    pub is_focused: bool,
    pub workspace: Option<String>,
    pub geometry: Option<WindowGeometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ProcessDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessDetails {
    pub user: Option<String>,
    pub exe: Option<String>,
    pub cmd: Option<Vec<String>>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Wayland,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compositor {
    X11,
    Wayland,
    Fht,
}

impl Compositor {
    pub async fn try_into_provider(&self) -> Result<Box<dyn WindowProvider>, WindowTrackerError> {
        match self {
            Compositor::Wayland => Ok(Box::new(WaylandProvider::try_new().await?)),
            Compositor::Fht => Ok(Box::new(FhtProvider::try_new().await?)),
            Compositor::X11 => Ok(Box::new(X11Provider::try_new()?)),
        }
    }
}

#[async_trait]
#[allow(dead_code)]
pub trait WindowProvider: Send + Sync {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError>;
    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError>;
    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError>;
    fn get_compositor(&self) -> Compositor;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WindowEvent {
    WindowFocused(WindowInfo),
    WindowOpened(WindowInfo),
    WindowClosed(String),
    WindowTitleChanged(WindowInfo),
}

pub struct WindowTracker {
    provider: Box<dyn WindowProvider>,
}

async fn find_window_provider() -> Result<Box<dyn WindowProvider>, WindowTrackerError> {
    // The orders of compositors is important since there may
    // be detection conflict, e.g. x11 detected on wayland
    let providers_to_try = [Compositor::Fht, Compositor::Wayland, Compositor::X11];

    for compositor in providers_to_try {
        match compositor.try_into_provider().await {
            Ok(provider) => return Ok(provider),
            Err(e) => {
                tracing::debug!("skipping compositor {compositor:?} due to error: {e:?}");
            }
        }
    }

    // If the loop finishes without returning, none of them worked.
    Err(WindowTrackerError::UnsupportedDisplayServer)
}

#[allow(dead_code)]
impl WindowTracker {
    pub async fn try_new() -> Result<Self, WindowTrackerError> {
        let window_provider = find_window_provider().await?;

        tracing::info!(
            "Using display server: {:?}",
            window_provider.get_compositor()
        );

        Ok(Self {
            provider: window_provider,
        })
    }

    pub async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        self.provider.get_active_window().await
    }

    pub async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        self.provider.get_all_windows().await
    }

    pub async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        self.provider.start_monitoring().await
    }

    pub fn get_compositor(&self) -> Compositor {
        self.provider.get_compositor()
    }
}
