use super::task;
use futures_util::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use window_tracker::WindowTracker;
use window_tracker::WindowTrackerError;

mod window_tracker;

#[derive(Clone)]
pub struct WindowEventService {
    task_service: task::TaskService,
}

impl WindowEventService {
    pub fn new(task_service: task::TaskService) -> Self {
        Self { task_service }
    }

    pub async fn start(&self) -> eyre::Result<()> {
        let tracker = match WindowTracker::new().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to initialize WindowTracker: {}", e);
                if let WindowTrackerError::UnsupportedCompositor(_) = e {
                    error!("\nThis windowing environment is not yet supported.");
                    error!("Supported environments include: X11, Sway/wlroots, Hyprland, KDE Plasma, Niri, and fht-compositor.");
                    error!(
                        "For others like GNOME, a stable API for window tracking is not available."
                    );
                }
                return Err(e.into());
            }
        };

        info!("Display server: {:?}", tracker.get_display_server());

        // Get active window
        match tracker.get_active_window().await {
            Ok(Some(window)) => info!("Active window: {:#?}", window),
            Ok(None) => info!("No active window found."),
            Err(e) => error!("Error getting active window: {}", e),
        }

        // Get all windows
        match tracker.get_all_windows().await {
            Ok(windows) => {
                info!("Found {} windows", windows.len());
                for w in windows.iter().take(5) {
                    // Print first 5
                    info!("- Window: {:#?}", w);
                }
            }
            Err(e) => error!("Error getting all windows: {}", e),
        }

        info!("\nStarting to monitor window events for 20 seconds... (Focus different windows to see events)");
        match tracker.start_monitoring().await {
            Ok(mut events) => {
                let timeout = sleep(Duration::from_secs(20));
                tokio::pin!(timeout);

                loop {
                    tokio::select! {
                        Some(event) = events.recv() => {
                            info!("Window event: {:#?}", event);
                        }
                        _ = &mut timeout => {
                            info!("Monitoring finished.");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Could not start monitoring: {}", e);
            }
        }

        Ok(())
    }
}
