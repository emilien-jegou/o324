mod dbus;
mod task_manager;
mod task_prefix_repository;
mod task_repository;
mod window_events;

use crate::{
    config::Config,
    core::storage::Storage,
    services::{
        dbus::DbusService, task_manager::TaskManagerService,
        task_prefix_repository::TaskPrefixRepository, task_repository::TaskRepository,
        window_events::WindowEventService,
    },
};

#[allow(dead_code)]
pub struct AppState {
    pub dbus_service: dbus::DbusService,
    pub window_event_service: window_events::WindowEventService,
    pub config: Config,
}

pub fn build(storage: Storage, config: Config) -> eyre::Result<AppState> {
    let task_repository = TaskRepository::builder()
        .storage(storage.clone())
        .config(config.clone())
        .build();

    let task_prefix_repository = TaskPrefixRepository::new(storage.clone());

    let task_manager_service = TaskManagerService::builder()
        .task_service(task_repository)
        .task_prefix_repository(task_prefix_repository)
        .build();

    let dbus_service = DbusService::builder()
        .task_manager_service(task_manager_service.clone())
        .build();

    let window_event_service = WindowEventService::builder()
        .task_manager_service(task_manager_service)
        .build();

    Ok(AppState {
        dbus_service,
        window_event_service,
        config,
    })
}
