use std::sync::Arc;

use wrap_builder::wrap_builder;

use crate::{
    config::Config,
    core::storage::Storage,
    repositories::{task::TaskRepository, task_prefix::TaskPrefixRepository},
    services::{
        dbus::DbusService, storage_bridge::StorageBridgeService, task::TaskService,
        window_events::WindowEventService,
    },
};

#[allow(dead_code)]
#[wrap_builder(Arc)]
pub struct App {
    pub dbus_service: DbusService,
    pub window_event_service: WindowEventService,
    pub config: Config,
}

pub fn build(storage: Storage, config: Config) -> eyre::Result<App> {
    let task_repository = TaskRepository::builder()
        .storage(storage.clone())
        .config(config.clone())
        .build();

    let task_prefix_repository = TaskPrefixRepository::new(storage.clone());

    let task_service = TaskService::builder()
        .task_repository(task_repository)
        .task_prefix_repository(task_prefix_repository)
        .build();

    let storage_bridge_service = StorageBridgeService::builder()
        .storage(storage.clone())
        .build();

    let dbus_service = DbusService::builder()
        .task_service(task_service.clone())
        .storage_bridge_service(storage_bridge_service)
        .build();

    let window_event_service = WindowEventService::builder()
        .task_service(task_service)
        .build();

    Ok(App::builder()
        .dbus_service(dbus_service)
        .window_event_service(window_event_service)
        .config(config)
        .build())
}
