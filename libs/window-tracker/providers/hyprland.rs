use async_trait::async_trait;
use serde::Deserialize;
use std::env;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::utils::get_process_info;
use crate::{
    Compositor, WindowEvent, WindowGeometry, WindowInfo, WindowProvider, WindowTrackerError,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct HyprlandClient {
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
#[allow(dead_code)]
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

pub struct HyprlandProvider {}

impl HyprlandProvider {
    pub async fn new() -> Result<Self, WindowTrackerError> {
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
