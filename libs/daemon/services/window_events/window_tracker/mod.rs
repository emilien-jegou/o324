use async_trait::async_trait;
use futures_util::stream::StreamExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt, Window};
use x11rb::rust_connection::{ConnectError, ConnectionError, ReplyError, RustConnection};

// nix and libc imports
use nix::libc::uid_t;
use nix::unistd::{Pid, Uid, User};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue};

#[derive(Error, Debug)]
pub enum WindowTrackerError {
    #[error("Unsupported display server: {0}")]
    UnsupportedDisplayServer(String),
    #[error("Compositor not supported: {0}")]
    UnsupportedCompositor(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Permission denied - may require additional setup")]
    PermissionDenied,
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

/// Gathers detailed information about a process using its PID.
/// This function relies on the Linux-specific /proc filesystem.
fn get_process_info(pid_u32: u32) -> Option<ProcessDetails> {
    if pid_u32 == 0 {
        return None;
    }
    let pid = Pid::from_raw(pid_u32 as i32);
    let proc_path = format!("/proc/{pid}");

    // Get executable path
    let exe = fs::read_link(format!("{proc_path}/exe"))
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    // Get command line arguments
    let cmd = fs::read_to_string(format!("{proc_path}/cmdline"))
        .ok()
        .map(|s| {
            s.split('\0')
                .map(String::from)
                .filter(|s| !s.is_empty())
                .collect()
        });

    // Get current working directory
    let cwd = fs::read_link(format!("{proc_path}/cwd"))
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    // Get user
    let user = fs::read_to_string(format!("{proc_path}/status"))
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("Uid:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|uid_str| uid_str.parse::<uid_t>().ok())
                .and_then(|uid| User::from_uid(Uid::from_raw(uid)).ok())
                .flatten()
                .map(|u| u.name)
        });

    Some(ProcessDetails {
        user,
        exe,
        cmd,
        cwd,
    })
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
    River,
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
            _ => Err(WindowTrackerError::UnsupportedCompositor(format!(
                "{self:?} is not yet supported"
            ))),
        }
    }
}

#[async_trait]
pub trait WindowProvider: Send + Sync {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError>;
    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError>;
    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError>;
    fn get_compositor(&self) -> Compositor;
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    WindowFocused(WindowInfo),
    WindowOpened(WindowInfo),
    WindowClosed(String), // window id
    WindowTitleChanged { id: String, new_title: String },
}

pub struct WindowTracker {
    provider: Box<dyn WindowProvider>,
    display_server: DisplayServer,
}

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
            Compositor::Sway
            | Compositor::Hyprland
            | Compositor::River
            | Compositor::Niri
            | Compositor::Fht => DisplayServer::Wayland,
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

// --- X11 Backend (Shared Logic) ---

#[derive(Debug, Clone, thiserror::Error)]
#[error("X11 Init Error: {0}")]
pub struct X11InitError(String);

#[derive(Clone, Copy)]
struct X11Atoms {
    _NET_ACTIVE_WINDOW: xproto::Atom,
    _NET_CLIENT_LIST: xproto::Atom,
    _NET_WM_NAME: xproto::Atom,
    _NET_WM_PID: xproto::Atom,
    UTF8_STRING: xproto::Atom,
    WM_CLASS: xproto::Atom,
}

impl X11Atoms {
    fn intern_all(conn: &impl Connection) -> Result<Self, WindowTrackerError> {
        let atoms_to_intern = [
            "_NET_ACTIVE_WINDOW",
            "_NET_CLIENT_LIST",
            "_NET_WM_NAME",
            "_NET_WM_PID",
            "UTF8_STRING",
            "WM_CLASS",
        ];
        let cookies: Vec<_> = atoms_to_intern
            .iter()
            .map(|name| conn.intern_atom(false, name.as_bytes()))
            .collect();
        let mut atoms = Vec::new();
        for cookie in cookies {
            atoms.push(cookie?.reply()?.atom);
        }
        Ok(Self {
            _NET_ACTIVE_WINDOW: atoms[0],
            _NET_CLIENT_LIST: atoms[1],
            _NET_WM_NAME: atoms[2],
            _NET_WM_PID: atoms[3],
            UTF8_STRING: atoms[4],
            WM_CLASS: atoms[5],
        })
    }
}

static X11_CONNECTION: Lazy<Result<(Arc<RustConnection>, usize), X11InitError>> = Lazy::new(|| {
    x11rb::connect(None)
        .map(|(conn, screen_num)| (Arc::new(conn), screen_num))
        .map_err(|e| X11InitError(e.to_string()))
});

#[derive(Clone)]
struct X11Backend {
    conn: Arc<RustConnection>,
    atoms: X11Atoms,
    root: Window,
}

impl X11Backend {
    fn new() -> Result<Self, WindowTrackerError> {
        let (conn, screen_num) = X11_CONNECTION.as_ref().map_err(Clone::clone)?.clone();
        let atoms = X11Atoms::intern_all(&*conn)?;
        let root = conn.setup().roots[screen_num].root;
        Ok(Self { conn, atoms, root })
    }

    fn get_window_info(
        &self,
        window: Window,
        is_focused: bool,
    ) -> Result<Option<WindowInfo>, WindowTrackerError> {
        if window == x11rb::NONE {
            return Ok(None);
        }

        let title_cookie = self.conn.get_property(
            false,
            window,
            self.atoms._NET_WM_NAME,
            self.atoms.UTF8_STRING,
            0,
            u32::MAX,
        )?;
        let class_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.WM_CLASS,
            xproto::AtomEnum::STRING,
            0,
            u32::MAX,
        )?;
        let pid_cookie = self.conn.get_property(
            false,
            window,
            self.atoms._NET_WM_PID,
            xproto::AtomEnum::CARDINAL,
            0,
            1,
        )?;
        let geom_cookie = self.conn.get_geometry(window)?;

        let title = String::from_utf8(title_cookie.reply()?.value).unwrap_or_default();
        let class_bytes = class_cookie.reply()?.value;
        let app_name =
            String::from_utf8_lossy(class_bytes.split(|&b| b == 0).nth(1).unwrap_or(b""))
                .to_string();

        let pid = pid_cookie.reply()?.value32().and_then(|mut i| i.next());
        let geom = geom_cookie.reply()?;
        let details = pid.and_then(get_process_info);

        Ok(Some(WindowInfo {
            id: window.to_string(),
            title,
            app_name,
            pid,
            is_focused,
            workspace: None,
            geometry: Some(WindowGeometry {
                x: geom.x as i32,
                y: geom.y as i32,
                width: geom.width as u32,
                height: geom.height as u32,
            }),
            details,
        }))
    }

    async fn get_active_window_backend(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let prop = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms._NET_ACTIVE_WINDOW,
                xproto::AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()?;
        let active_window = prop
            .value32()
            .and_then(|mut v| v.next())
            .unwrap_or(x11rb::NONE);
        self.get_window_info(active_window, true)
    }

    async fn get_all_windows_backend(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let active_window = self.get_active_window_backend().await?.map(|w| w.id);
        let prop = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms._NET_CLIENT_LIST,
                xproto::AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?;
        let mut windows = Vec::new();
        if let Some(client_list) = prop.value32() {
            for win_id in client_list {
                let is_focused = active_window.as_deref() == Some(&win_id.to_string());
                if let Ok(Some(info)) = self.get_window_info(win_id, is_focused) {
                    windows.push(info);
                }
            }
        }
        Ok(windows)
    }
}

// --- EWMH Provider (Generic X11 Fallback) ---
struct EwmhProvider {
    backend: X11Backend,
}

impl EwmhProvider {
    fn new() -> Result<Self, WindowTrackerError> {
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
                        error!("Polling for active X11 window failed: {}", e);
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
// --- Sway Provider ---
#[derive(Debug, Deserialize)]
struct SwayNode {
    id: i64,
    name: Option<String>,
    app_id: Option<String>,
    pid: Option<u32>,
    focused: bool,
    rect: SwayRect,
    #[serde(default)]
    nodes: Vec<SwayNode>,
    #[serde(default)]
    floating_nodes: Vec<SwayNode>,
    // other fields omitted
}

#[derive(Debug, Deserialize)]
struct SwayRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl WindowInfo {
    fn from_sway_node(node: &SwayNode) -> Option<Self> {
        let details = node.pid.and_then(get_process_info);
        Some(Self {
            id: node.id.to_string(),
            title: node.name.clone().unwrap_or_default(),
            app_name: node
                .app_id
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_default(),
            pid: node.pid,
            is_focused: node.focused,
            workspace: None,
            geometry: Some(WindowGeometry {
                x: node.rect.x,
                y: node.rect.y,
                width: node.rect.width,
                height: node.rect.height,
            }),
            details,
        })
    }
}

#[derive(Deserialize)]
struct SwayEvent {
    change: String,
    container: SwayNode,
}

impl WindowEvent {
    fn from_sway_event(event: &SwayEvent) -> Option<Self> {
        match event.change.as_str() {
            "focus" | "new" | "title" => {
                let info = WindowInfo::from_sway_node(&event.container)?;
                match event.change.as_str() {
                    "focus" => Some(WindowEvent::WindowFocused(info)),
                    "new" => Some(WindowEvent::WindowOpened(info)),
                    "title" => Some(WindowEvent::WindowTitleChanged {
                        id: info.id,
                        new_title: info.title,
                    }),
                    _ => None,
                }
            }
            "close" => Some(WindowEvent::WindowClosed(event.container.id.to_string())),
            _ => None,
        }
    }
}

struct SwayProvider {}

impl SwayProvider {
    async fn new() -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        Ok(Self {})
    }

    pub async fn detect() -> Option<Compositor> {
        if env::var("SWAYSOCK").is_ok() {
            Some(Compositor::Sway)
        } else {
            None
        }
    }

    async fn execute_sway_command(&self, args: &[&str]) -> Result<String, WindowTrackerError> {

        let output = Command::new("swaymsg")
            .args(args)
            .output()
            .await
            .map_err(|e| WindowTrackerError::CommandFailed(e.to_string()))?;
        if !output.status.success() {
            return Err(WindowTrackerError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn find_focused_node(node: &SwayNode) -> Option<&SwayNode> {
        if node.focused {
            return Some(node);
        }
        node.nodes
            .iter()
            .find_map(Self::find_focused_node)
            .or_else(|| node.floating_nodes.iter().find_map(Self::find_focused_node))
    }

    fn collect_windows(node: &SwayNode, windows: &mut Vec<WindowInfo>) {
        if node.pid.is_some() {
            if let Some(info) = WindowInfo::from_sway_node(node) {
                windows.push(info);
            }
        }
        for child in &node.nodes {
            Self::collect_windows(child, windows);
        }
        for child in &node.floating_nodes {
            Self::collect_windows(child, windows);
        }
    }
}

#[async_trait]
impl WindowProvider for SwayProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let output = self.execute_sway_command(&["-t", "get_tree"]).await?;
        let tree: SwayNode = serde_json::from_str(&output)
            .map_err(|e| WindowTrackerError::ParseError(format!("Sway get_tree: {e}")))?;
        Ok(Self::find_focused_node(&tree).and_then(WindowInfo::from_sway_node))
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let output = self.execute_sway_command(&["-t", "get_tree"]).await?;
        let tree: SwayNode = serde_json::from_str(&output)
            .map_err(|e| WindowTrackerError::ParseError(format!("Sway get_tree: {e}")))?;
        let mut windows = Vec::new();
        Self::collect_windows(&tree, &mut windows);
        Ok(windows)
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let mut cmd = Command::new("swaymsg")
            .args(["-m", "-t", "subscribe", r#"["window"]"#])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| {
                WindowTrackerError::CommandFailed(format!("Failed to start swaymsg: {e}"))
            })?;

        let stdout = cmd.stdout.take().expect("stdout should be available");

        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                match serde_json::from_str::<SwayEvent>(&line) {
                    Ok(event) => {
                        if let Some(window_event) = WindowEvent::from_sway_event(&event) {
                            if tx.send(window_event).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => warn!("Failed to parse sway event: {}", e),
                }
            }
            cmd.kill().await.ok();
        });
        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Sway
    }
}

// --- Hyprland Provider ---
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HyprlandClient {
    address: String,
    at: (i32, i32),
    size: (u32, u32),
    workspace: HyprlandWorkspace,
    class: String,
    title: String,
    pid: i64,
    focus_history_id: i32,
}

#[derive(Debug, Deserialize)]
struct HyprlandWorkspace {
    id: i32,
    name: String,
}

impl WindowInfo {
    fn from_hyprland_client(client: &HyprlandClient, is_focused: bool) -> Self {
        let pid = if client.pid > 0 {
            Some(client.pid as u32)
        } else {
            None
        };
        let details = pid.and_then(get_process_info);
        Self {
            id: client.address.clone(),
            title: client.title.clone(),
            app_name: client.class.clone(),
            pid,
            is_focused,
            workspace: Some(client.workspace.name.clone()),
            geometry: Some(WindowGeometry {
                x: client.at.0,
                y: client.at.1,
                width: client.size.0,
                height: client.size.1,
            }),
            details,
        }
    }
}

struct HyprlandProvider {}
impl HyprlandProvider {
    async fn new() -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        Ok(Self {})
    }

    pub async fn detect() -> Option<Compositor> {
        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            Some(Compositor::Hyprland)
        } else {
            None
        }
    }

    async fn execute_hyprctl_command(&self, args: &[&str]) -> Result<String, WindowTrackerError> {
        let output = Command::new("hyprctl")
            .args(args)
            .output()
            .await
            .map_err(|e| WindowTrackerError::CommandFailed(e.to_string()))?;
        if !output.status.success() {
            return Err(WindowTrackerError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[async_trait]
impl WindowProvider for HyprlandProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let output = self
            .execute_hyprctl_command(&["-j", "activewindow"])
            .await?;
        if output.trim().is_empty() {
            return Ok(None);
        }
        let client: HyprlandClient = serde_json::from_str(&output)
            .map_err(|e| WindowTrackerError::ParseError(format!("Hyprland activewindow: {e}")))?;
        Ok(Some(WindowInfo::from_hyprland_client(&client, true)))
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let output = self.execute_hyprctl_command(&["-j", "clients"]).await?;
        let clients: Vec<HyprlandClient> = serde_json::from_str(&output)
            .map_err(|e| WindowTrackerError::ParseError(format!("Hyprland clients: {e}")))?;
        let active_window_id = self.get_active_window().await?.map(|w| w.id);
        Ok(clients
            .into_iter()
            .map(|c| {
                WindowInfo::from_hyprland_client(
                    &c,
                    active_window_id.as_deref() == Some(&c.address),
                )
            })
            .collect())
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| {
            WindowTrackerError::CommandFailed("HYPRLAND_INSTANCE_SIGNATURE not set".to_string())
        })?;
        let socket_path = format!("/tmp/hypr/{signature}/.socket2.sock");

        let stream = tokio::net::UnixStream::connect(socket_path)
            .await
            .map_err(|e| {
                WindowTrackerError::CommandFailed(format!(
                    "Failed to connect to hyprland event socket: {e}"
                ))
            })?;

        tokio::spawn(async move {
            let mut lines = BufReader::new(stream).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let parts: Vec<_> = line.splitn(2, ">>").collect();
                if parts.len() < 2 {
                    continue;
                }
                let event_type = parts[0];
                let _event_data = parts[1];

                let event = match event_type {
                    "activewindow" => {
                        let provider = HyprlandProvider::new().await.unwrap();
                        if let Ok(Some(info)) = provider.get_active_window().await {
                            Some(WindowEvent::WindowFocused(info))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(e) = event {
                    if tx.send(e).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Hyprland
    }
}

// --- Gnome Provider ---
struct GnomeProvider {
    backend: GnomeBackend,
    display_server: DisplayServer,
}

enum GnomeBackend {
    X11(X11Backend),
    Wayland, // Gnome on Wayland has no stable public API for window info
}

impl GnomeProvider {
    async fn new(display_server: DisplayServer) -> Result<Self, WindowTrackerError> {
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
                            if last_focused.as_ref().map_or(true, |l| l.id != current.id) {
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

// --- KDE Provider ---
struct KdeProvider {
    conn: Arc<Mutex<zbus::Connection>>,
    display_server: DisplayServer,
}

impl KdeProvider {
    async fn new(display_server: DisplayServer) -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        let conn = zbus::Connection::session().await?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            display_server,
        })
    }

    pub async fn detect() -> Option<Compositor> {
        let is_kde = env::var("XDG_CURRENT_DESKTOP")
            .map_or(false, |d| d.to_lowercase().contains("kde"))
            || env::var("KDE_FULL_SESSION").is_ok();

        if is_kde {
            if env::var("WAYLAND_DISPLAY").is_ok() {
                return Some(Compositor::Kde(DisplayServer::Wayland));
            }
            if env::var("DISPLAY").is_ok() {
                return Some(Compositor::Kde(DisplayServer::X11));
            }
        }
        None
    }

    async fn get_info_from_client(
        &self,
        client_id: u64,
        is_focused: bool,
    ) -> Result<WindowInfo, WindowTrackerError> {
        let conn = self.conn.lock().await;
        let path_str = format!("/client/{client_id}");
        let path = ObjectPath::try_from(path_str).unwrap();

        let props_proxy = zbus::Proxy::new(
            &conn,
            "org.kde.KWin",
            &path,
            "org.freedesktop.DBus.Properties",
        )
        .await?;
        let props: HashMap<String, OwnedValue> = props_proxy
            .call("GetAll", &("org.kde.kwin.Client",))
            .await?;

        let title: String = props
            .get("caption")
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or_default();
        let app_name: String = props
            .get("resourceClass")
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or_default();
        let pid: u32 = props
            .get("pid")
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or(0);
        let pid_opt = if pid > 0 { Some(pid) } else { None };
        let details = pid_opt.and_then(get_process_info);

        Ok(WindowInfo {
            id: client_id.to_string(),
            title,
            app_name,
            pid: pid_opt,
            is_focused,
            workspace: None, // Geometry and workspace info is more complex
            geometry: None,
            details,
        })
    }
}

#[async_trait]
impl WindowProvider for KdeProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let conn = self.conn.lock().await;
        let proxy = zbus::Proxy::new(&conn, "org.kde.KWin", "/KWin", "org.kde.KWin").await?;
        let client_id: u64 = proxy.get_property("activeClient").await?;
        if client_id == 0 {
            return Ok(None);
        }
        Ok(Some(self.get_info_from_client(client_id, true).await?))
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let conn = self.conn.lock().await;
        let proxy = zbus::Proxy::new(&conn, "org.kde.KWin", "/KWin", "org.kde.KWin").await?;
        let client_ids: Vec<u64> = proxy.get_property("clientList").await?;
        let active_client_id = proxy.get_property::<u64>("activeClient").await.unwrap_or(0);

        let mut windows = Vec::new();
        for id in client_ids {
            if let Ok(info) = self.get_info_from_client(id, id == active_client_id).await {
                windows.push(info);
            }
        }
        Ok(windows)
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let conn = self.conn.lock().await;
        let proxy = zbus::Proxy::new(&conn, "org.kde.KWin", "/KWin", "org.kde.KWin").await?;

        let mut stream = proxy.receive_signal("activeClientChanged").await?;
        let provider = Self {
            conn: Arc::clone(&self.conn),
            display_server: self.display_server.clone(),
        };

        tokio::spawn(async move {
            while let Some(signal) = stream.next().await {
                if let Ok(client_id) = signal.body().deserialize::<u64>() {
                    if client_id != 0 {
                        if let Ok(info) = provider.get_info_from_client(client_id, true).await {
                            if tx.send(WindowEvent::WindowFocused(info)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Kde(self.display_server.clone())
    }
}

// --- Niri Provider ---
struct NiriProvider {
    conn: Arc<Mutex<zbus::Connection>>,
}

impl NiriProvider {
    async fn new() -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        let conn = zbus::Connection::session().await?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn detect() -> Option<Compositor> {
        if let Ok(conn) = zbus::Connection::session().await {
            if let Ok(proxy) = zbus::Proxy::new(
                &conn,
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
            )
            .await
            {
                let result: Result<bool, _> = proxy.call("NameHasOwner", &("re.sonny.niri",)).await;
                if let Ok(true) = result {
                    return Some(Compositor::Niri);
                }
            }
        }
        None
    }

    async fn get_session_proxy(&self) -> Result<zbus::Proxy<'_>, WindowTrackerError> {
        let conn = self.conn.lock().await;
        Ok(zbus::Proxy::new(
            &conn,
            "re.sonny.niri",
            "/re/sonny/niri",
            "re.sonny.niri.Session",
        )
        .await?)
    }

    async fn get_info_from_window_path(
        &self,
        window_path: OwnedObjectPath,
        is_focused: bool,
    ) -> Result<WindowInfo, WindowTrackerError> {
        let conn = self.conn.lock().await;
        let props_proxy = zbus::Proxy::new(
            &conn,
            "re.sonny.niri",
            &window_path,
            "org.freedesktop.DBus.Properties",
        )
        .await?;
        let props: HashMap<String, OwnedValue> = props_proxy
            .call("GetAll", &("re.sonny.niri.Window",))
            .await?;

        let title: String = props
            .get("title")
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or_default();
        let app_id: String = props
            .get("app-id")
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or_default();
        let pid: u32 = props
            .get("pid")
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or(0);
        let pid_opt = if pid > 0 { Some(pid) } else { None };
        let details = pid_opt.and_then(get_process_info);

        Ok(WindowInfo {
            id: window_path.to_string(),
            title,
            app_name: app_id,
            pid: pid_opt,
            is_focused,
            workspace: None, // Niri has a different workspace concept
            geometry: None,  // Niri does not seem to expose this via DBus
            details,
        })
    }
}

#[async_trait]
impl WindowProvider for NiriProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let proxy = self.get_session_proxy().await?;
        let outputs: Vec<OwnedObjectPath> = proxy.get_property("outputs").await?;

        for output_path in outputs {
            let conn = self.conn.lock().await;
            let output_proxy =
                zbus::Proxy::new(&conn, "re.sonny.niri", &output_path, "re.sonny.niri.Output")
                    .await?;
            if let Ok(true) = output_proxy.get_property::<bool>("focused").await {
                if let Ok(window_path) = output_proxy
                    .get_property::<OwnedObjectPath>("focused_window")
                    .await
                {
                    return Ok(Some(
                        self.get_info_from_window_path(window_path, true).await?,
                    ));
                }
            }
        }
        Ok(None)
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let proxy = self.get_session_proxy().await?;
        let outputs: Vec<OwnedObjectPath> = proxy.get_property("outputs").await?;
        let mut all_windows = Vec::new();

        if let Ok(Some(active_win)) = self.get_active_window().await {
            let active_id = active_win.id.clone();
            all_windows.push(active_win);

            for output_path in outputs {
                let conn = self.conn.lock().await;
                let output_proxy =
                    zbus::Proxy::new(&conn, "re.sonny.niri", &output_path, "re.sonny.niri.Output")
                        .await?;
                let window_paths: Vec<OwnedObjectPath> =
                    output_proxy.get_property("windows").await?;

                for window_path in window_paths {
                    if window_path.as_str() != active_id {
                        if let Ok(info) = self.get_info_from_window_path(window_path, false).await {
                            all_windows.push(info);
                        }
                    }
                }
            }
        }
        Ok(all_windows)
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let proxy = self.get_session_proxy().await?;

        let mut stream = proxy.receive_signal("active_window_changed").await?;
        let provider = Self {
            conn: Arc::clone(&self.conn),
        };

        tokio::spawn(async move {
            while let Some(signal) = stream.next().await {
                if let Ok(window_path) = signal.body().deserialize::<OwnedObjectPath>() {
                    if let Ok(info) = provider.get_info_from_window_path(window_path, true).await {
                        if tx.send(WindowEvent::WindowFocused(info)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Niri
    }
}

// --- FHT Provider ---
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct FhtWindow {
    id: u64,
    title: String,
    app_id: String,
    size: (u32, u32),
    location: (i32, i32),
    focused: bool,
}

impl WindowInfo {
    fn from_fht_window(fht_win: &FhtWindow) -> Self {
        // FHT IPC does not provide a PID, so process details will be None.
        Self {
            id: fht_win.id.to_string(),
            title: fht_win.title.clone(),
            app_name: fht_win.app_id.clone(),
            pid: None,
            is_focused: fht_win.focused,
            workspace: None, // Workspace info could be constructed, but keeping it simple.
            geometry: Some(WindowGeometry {
                x: fht_win.location.0,
                y: fht_win.location.1,
                width: fht_win.size.0,
                height: fht_win.size.1,
            }),
            details: None,
        }
    }
}

#[derive(Clone)]
struct FhtProvider;

impl FhtProvider {
    async fn new() -> Result<Self, WindowTrackerError> {
        Ok(Self {})
    }

    pub async fn detect() -> Option<Compositor> {
        let desktop_session = env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();

        if desktop_session == "fht-compositor" {
            Some(Compositor::Fht)
        } else {
            None
        }
    }

    async fn execute_fht_command(&self, args: &[&str]) -> Result<String, WindowTrackerError> {
        // Prepend the required 'ipc' and '-j' flags for JSON output.
        let mut full_args = vec!["ipc", "-j"];
        full_args.extend(args);

        let output = Command::new("fht-compositor")
            .args(&full_args)
            .output()
            .await
            .map_err(|e| WindowTrackerError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(WindowTrackerError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[async_trait]
impl WindowProvider for FhtProvider {
    async fn get_active_window(&self) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let output = self.execute_fht_command(&["focused-window"]).await?;
        if output.trim().is_empty() || output.trim() == "null" {
            return Ok(None);
        }
        let fht_win: FhtWindow = serde_json::from_str(&output).map_err(|e| {
            WindowTrackerError::ParseError(format!("fht-compositor focused-window: {e}"))
        })?;
        Ok(Some(WindowInfo::from_fht_window(&fht_win)))
    }

    async fn get_all_windows(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let output = self.execute_fht_command(&["windows"]).await?;
        let fht_windows: Vec<FhtWindow> = serde_json::from_str(&output)
            .map_err(|e| WindowTrackerError::ParseError(format!("fht-compositor windows: {e}")))?;
        let windows = fht_windows
            .iter()
            .map(WindowInfo::from_fht_window)
            .collect();
        Ok(windows)
    }

    async fn start_monitoring(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<WindowEvent>, WindowTrackerError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let provider = self.clone();

        tokio::spawn(async move {
            let mut last_focused_window: Option<WindowInfo> = None;
            loop {
                match provider.get_active_window().await {
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
                        error!("Polling for active fht-compositor window failed: {}", e);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
                sleep(Duration::from_millis(200)).await;
            }
        });

        Ok(rx)
    }

    fn get_compositor(&self) -> Compositor {
        Compositor::Fht
    }
}
