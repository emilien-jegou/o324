use crate::core::utils::unix_now_ms;
use crate::entities::activity::Activity;
use crate::repositories::activity::defs::StartActivity;
use crate::repositories::activity::ActivityRepository;
use crate::services::task::TaskService;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
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

#[derive(Clone)]
struct LastActiveWindowData {
    pub window_id: String,
    pub activity_id: String,
    pub last_active: u64,
}

#[wrap_builder(Arc)]
#[allow(dead_code)]
pub struct ActivityService {
    task_service: TaskService,
    activity_repository: ActivityRepository,
    #[builder(default)]
    last_active_window: RwLock<Option<LastActiveWindowData>>,
}

async fn build_window_tracker() -> Result<WindowTracker> {
    WindowTracker::try_new().await.map_err(|e| {
        error!("Failed to initialize WindowTracker: {}", e);
        if let WindowTrackerError::UnsupportedCompositor(_) = &e {
            error!("\nThis windowing environment is not yet supported.");
        }
        Error::TrackerInitialization(e)
    })
}

impl ActivityServiceInner {
    async fn handle_window_event(&self, event: WindowEvent) -> eyre::Result<()> {
        info!("Window event: {:#?}", event);
        let info = match event {
            WindowEvent::WindowFocused(info) | WindowEvent::WindowTitleChanged(info) => info,
            _ => return Ok(()),
        };

        let now = unix_now_ms();
        let mut write_window = self.last_active_window.write().await;
        if let Some(mut activity) = (*write_window).clone() {
            if activity.window_id == info.id {
                self.activity_repository
                    .update_last_event(activity.activity_id.clone(), now)?;
                activity.last_active = now;
                *write_window = Some(activity);
                return Ok(());
            }
            // We join the last application if the time gap is less than 1 minutes for
            // time-keeping accuracy.
            else if now - activity.last_active < 1000 * 60 * 1 {
                self.activity_repository
                    .update_last_event(activity.activity_id.clone(), now)?;
            }
            // ... we also add one minutes to the last_activity date, again
            // for time-keeping accuracy.
            else {
                self.activity_repository.update_last_event(
                    activity.activity_id.clone(),
                    activity.last_active + 1000 * 60 * 1,
                )?;
            }
        }

        let activity_id = self.activity_repository.register(StartActivity {
            app_name: info.app_name.clone(),
            at: now,
        })?;

        *write_window = Some(LastActiveWindowData {
            window_id: info.id,
            activity_id,
            last_active: now,
        });

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
