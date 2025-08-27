use async_trait::async_trait;
use serde::Deserialize;
use std::env;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

use crate::{
    Compositor, WindowEvent, WindowGeometry, WindowInfo, WindowProvider, WindowTrackerError,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FhtWindow {
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
pub struct FhtProvider;

impl FhtProvider {
    pub async fn new() -> Result<Self, WindowTrackerError> {
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
                        tracing::error!("Polling for active fht-compositor window failed: {}", e);
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
