mod dbus;
mod task;
mod window_events;

use crate::{config::Config, core::storage::Storage};

#[allow(dead_code)]
pub struct AppState {
    pub task_service: task::TaskService,
    pub dbus_service: dbus::DbusService,
    pub window_event_service: window_events::WindowEventService,
    pub config: Config,
}

pub fn build(storage: Storage, config: Config) -> eyre::Result<AppState> {
    let task_service = task::TaskService::try_new(storage, config.clone())?;

    let dbus_service = dbus::DbusService::new(task_service.clone());
    let window_event_service = window_events::WindowEventService::new(task_service.clone());

    Ok(AppState {
        task_service,
        dbus_service,
        window_event_service,
        config,
    })
}
