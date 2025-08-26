use async_trait::async_trait;
use serde::Deserialize;
use std::env;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::warn;

use crate::services::window_events::window_tracker::utils::get_process_info;
use crate::services::window_events::window_tracker::{
    Compositor, WindowEvent, WindowGeometry, WindowInfo, WindowProvider, WindowTrackerError,
};
pub struct SwayProvider {}

impl SwayProvider {
    pub async fn new() -> Result<Self, WindowTrackerError> {
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
