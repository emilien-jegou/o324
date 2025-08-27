use async_trait::async_trait;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zvariant::{OwnedObjectPath, OwnedValue};

use crate::{
    utils::get_process_info, Compositor, WindowEvent, WindowInfo, WindowProvider,
    WindowTrackerError,
};

// --- Niri Provider ---
pub struct NiriProvider {
    conn: Arc<Mutex<zbus::Connection>>,
}

impl NiriProvider {
    pub async fn new() -> Result<Self, WindowTrackerError> {
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
