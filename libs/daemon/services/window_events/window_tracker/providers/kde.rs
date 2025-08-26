use async_trait::async_trait;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use zvariant::{ObjectPath, OwnedValue};

use crate::services::window_events::window_tracker::{
    utils::get_process_info, Compositor, DisplayServer, WindowEvent, WindowInfo, WindowProvider,
    WindowTrackerError,
};

// --- KDE Provider ---
pub struct KdeProvider {
    conn: Arc<Mutex<zbus::Connection>>,
    display_server: DisplayServer,
}

impl KdeProvider {
    pub async fn new(display_server: DisplayServer) -> Result<Self, WindowTrackerError> {
        tracing::warn!("window detection is experimental for this compositor");
        let conn = zbus::Connection::session().await?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            display_server,
        })
    }

    pub async fn detect() -> Option<Compositor> {
        let is_kde = env::var("XDG_CURRENT_DESKTOP")
            .is_ok_and(|d| d.to_lowercase().contains("kde"))
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
