use std::sync::Arc;

use crate::services::task_manager::TaskManagerService;

use thiserror::Error; // 1. Import thiserror
use tracing::{error, info};
use window_tracker::WindowTracker;
use window_tracker::WindowTrackerError;
use wrap_builder::wrap_builder;

// 2. Define a custom error enum for this service
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize the window tracker")]
    TrackerInitialization(#[from] WindowTrackerError),

    #[error("Failed to start monitoring window events")]
    MonitoringStart(WindowTrackerError),
    // You could add other service-specific errors here in the future
}

// 3. Create a convenient Result type alias
pub type Result<T> = std::result::Result<T, Error>;

#[wrap_builder(Arc)]
#[allow(dead_code)]
pub struct WindowEventService {
    task_manager_service: TaskManagerService,
}

// 4. Update the function signature to return our custom Result
async fn build_window_tracker() -> Result<WindowTracker> {
    WindowTracker::new().await.map_err(|e| {
      error!("Failed to initialize WindowTracker: {}", e);
      if let WindowTrackerError::UnsupportedCompositor(_) = &e {
          error!("\nThis windowing environment is not yet supported.");
          error!("Supported environments include: X11, Sway/wlroots, Hyprland, KDE Plasma, Niri, and fht-compositor.");
          error!(
              "For others like GNOME, a stable API for window tracking is not available."
          );
      }
      // 5. Map the underlying error to our new custom error variant.
      // The `#[from]` attribute on the TrackerInitialization variant
      // allows us to use `?` or `.into()` for this conversion.
      Error::TrackerInitialization(e)
    })
}

impl WindowEventService {
    /// Starts the window event monitoring service.
    // 6. Update the method signature
    pub async fn start(&self) -> Result<()> {
        let window_tracker = build_window_tracker().await?;
        info!("Display server: {:?}", window_tracker.get_display_server());
        info!("Compositor: {:?}", window_tracker.get_compositor());

        match window_tracker.get_active_window().await {
            Ok(Some(window)) => info!("Initial active window: {}", window.title),
            Ok(None) => info!("No active window found on startup."),
            Err(e) => error!("Error getting initial active window: {}", e),
        }

        info!("Starting window events monitoring.");
        let mut events = window_tracker.start_monitoring().await.map_err(|e| {
            error!("Could not start window monitoring: {}", e);
            // 7. Map this specific error to our MonitoringStart variant
            Error::MonitoringStart(e)
        })?;

        // Loop indefinitely, processing events as they come in.
        while let Some(event) = events.recv().await {
            info!("Window event: {:#?}", event);
            // Here you can use self.task_service to process the event,
            // for example:
            // self.task_service.handle_window_event(event).await;
        }

        info!("Window event monitoring stream has ended.");

        Ok(())
    }
}
