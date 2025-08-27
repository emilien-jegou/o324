use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info};
use x11rb::rust_connection::{ConnectError, ConnectionError, ReplyError};

pub mod providers;
pub mod utils;
pub mod x11;

use x11::X11InitError;

use providers::{
    ewmh::EwmhProvider, fht_compositor::FhtProvider, gnome::GnomeProvider,
    hyprland::HyprlandProvider, kde::KdeProvider, niri::NiriProvider, sway::SwayProvider,
};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum WindowTrackerError {
    #[error("Unsupported display server: {0}")]
    UnsupportedDisplayServer(String),
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
    Sway,
    Gnome(DisplayServer),
    Kde(DisplayServer),
    Hyprland,
    Niri,
    Fht,
    Unknown,
}

impl Compositor {
    pub async fn into_provider(self) -> Result<Box<dyn WindowProvider>, WindowTrackerError> {
        match self {
            Compositor::Sway => Ok(Box::new(SwayProvider::new().await?)),
            Compositor::Hyprland => Ok(Box::new(HyprlandProvider::new().await?)),
            Compositor::Niri => Ok(Box::new(NiriProvider::new().await?)),
            Compositor::Fht => Ok(Box::new(FhtProvider::new().await?)),
            Compositor::Gnome(ds) => Ok(Box::new(GnomeProvider::new(ds).await?)),
            Compositor::Kde(ds) => Ok(Box::new(KdeProvider::new(ds).await?)),
            Compositor::Unknown => Ok(Box::new(EwmhProvider::new()?)),
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
    WindowTitleChanged { id: String, new_title: String },
}

pub struct WindowTracker {
    provider: Box<dyn WindowProvider>,
    display_server: DisplayServer,
}

#[allow(dead_code)]
impl WindowTracker {
    pub async fn new() -> Result<Self, WindowTrackerError> {
        let compositor = Self::detect_environment().await.ok_or_else(|| {
            WindowTrackerError::UnsupportedDisplayServer(
                "Could not detect a supported display server or compositor.".to_string(),
            )
        })?;

        let provider = compositor.clone().into_provider().await?;

        let display_server = match &compositor {
            Compositor::Gnome(ds) | Compositor::Kde(ds) => ds.clone(),
            Compositor::Sway | Compositor::Hyprland | Compositor::Niri | Compositor::Fht => {
                DisplayServer::Wayland
            }
            Compositor::Unknown => DisplayServer::X11,
        };

        info!(
            "Detected display server: {:?}, compositor: {:?}",
            display_server, &compositor
        );

        Ok(Self {
            provider,
            display_server,
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

    pub fn get_display_server(&self) -> &DisplayServer {
        &self.display_server
    }

    pub fn get_compositor(&self) -> Compositor {
        self.provider.get_compositor()
    }

    async fn detect_environment() -> Option<Compositor> {
        // Chain provider detection methods. Order is important: more specific checks first.
        if let Some(c) = HyprlandProvider::detect().await {
            return Some(c);
        }
        if let Some(c) = SwayProvider::detect().await {
            return Some(c);
        }
        if let Some(c) = NiriProvider::detect().await {
            return Some(c);
        }
        if let Some(c) = FhtProvider::detect().await {
            return Some(c);
        }
        if let Some(c) = GnomeProvider::detect().await {
            return Some(c);
        }
        if let Some(c) = KdeProvider::detect().await {
            return Some(c);
        }
        // X11 Fallback for other EWMH-compliant WMs
        if let Some(c) = EwmhProvider::detect().await {
            return Some(c);
        }

        None
    }
}
