use crate::core::utils::unix_now;
use crate::entities::activity::Activity;
use crate::repositories::activity::defs::StartActivity;
use crate::repositories::activity::ActivityRepository;
use crate::services::task::TaskService;
use std::sync::Arc;

use thiserror::Error;
use tracing::{error, info};
use window_tracker::WindowEvent;
use window_tracker::WindowTracker;
use window_tracker::WindowTrackerError;
use wrap_builder::wrap_builder;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize the window tracker")]
    TrackerInitialization(#[from] WindowTrackerError),

    #[error("Failed to start monitoring window events")]
    MonitoringStart(WindowTrackerError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[wrap_builder(Arc)]
#[allow(dead_code)]
pub struct ActivityService {
    task_service: TaskService,
    activity_repository: ActivityRepository,
}

async fn build_window_tracker() -> Result<WindowTracker> {
    WindowTracker::try_new().await.map_err(|e| {
      error!("Failed to initialize WindowTracker: {}", e);
      if let WindowTrackerError::UnsupportedCompositor(_) = &e {
          error!("\nThis windowing environment is not yet supported.");
          error!("Supported environments include: X11, Sway/wlroots, Hyprland, KDE Plasma, Niri, and fht-compositor.");
          error!(
              "For others like GNOME, a stable API for window tracking is not available."
          );
      }
      Error::TrackerInitialization(e)
    })
}

impl ActivityService {
    async fn handle_window_event(&self, event: WindowEvent) -> eyre::Result<()> {
        info!("Window event: {:#?}", event);
        match event {
            WindowEvent::WindowFocused(info) | WindowEvent::WindowTitleChanged(info) => {
                self.activity_repository.register(StartActivity {
                    app_title: Some(info.title.clone()),
                    app_name: info.app_name.clone(),
                    at: unix_now(),
                })?;
            }
            _ => (),
        }
        Ok(())
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let window_tracker = build_window_tracker().await?;

        match window_tracker.get_active_window().await {
            Ok(Some(window)) => info!("Initial active window: {}", window.title),
            Ok(None) => info!("No active window found on startup."),
            Err(e) => error!("Error getting initial active window: {}", e),
        }

        info!("Starting window events monitoring.");
        let mut events = window_tracker.start_monitoring().await.map_err(|e| {
            error!("Could not start window monitoring: {}", e);
            Error::MonitoringStart(e)
        })?;

        while let Some(event) = events.recv().await {
            if let Err(err) = self.handle_window_event(event).await {
                tracing::warn!("An error occured while handling a window event: {err}");
            }
        }

        info!("Window event monitoring stream has ended.");
        Ok(())
    }

    pub async fn list_activity_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> eyre::Result<Vec<Activity>> {
        Ok(self
            .activity_repository
            .list_activity_range(start_timestamp, end_timestamp)
            .await?)
    }
}
