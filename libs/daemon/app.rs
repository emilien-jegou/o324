use std::sync::Arc;
use wrap_builder::wrap_builder;

use crate::{
    actors::{activity::ActivityActor, dbus::DbusActor},
    config::Config,
    core::storage::Storage,
    repositories::{
        activity::ActivityRepository, project_color::ProjectColorRepository, task::TaskRepository,
        task_prefix::TaskPrefixRepository,
    },
    services::{
        activity::ActivityService, storage_bridge::StorageBridgeService, task::TaskService,
    },
};

#[allow(dead_code)]
#[wrap_builder(Arc)]
pub struct App {
    pub dbus_actor: DbusActor,
    pub activity_actor: ActivityActor,
    pub config: Config,
}

pub fn build(storage: Storage, config: Config) -> eyre::Result<App> {
    let task_repository = TaskRepository::builder()
        .storage(storage.clone())
        .computer_name(config.core.computer_name.clone())
        .build();

    let activity_repository = ActivityRepository::builder()
        .storage(storage.clone())
        .computer_name(config.core.computer_name.clone())
        .build();

    let task_prefix_repository = TaskPrefixRepository::new(storage.clone());

    let project_color_repository = ProjectColorRepository::builder()
        .storage(storage.clone())
        .build();

    let task_service = TaskService::builder()
        .task_repository(task_repository)
        .task_prefix_repository(task_prefix_repository)
        .project_color_repository(project_color_repository)
        .build();

    let storage_bridge_service = StorageBridgeService::builder()
        .storage(storage.clone())
        .build();

    let activity_service = ActivityService::builder()
        .task_service(task_service.clone())
        .activity_repository(activity_repository.clone())
        .build();

    let dbus_actor = DbusActor::builder()
        .task_service(task_service.clone())
        .activity_service(activity_service.clone())
        .storage_bridge_service(storage_bridge_service)
        .build();

    let activity_actor = ActivityActor::builder()
        .activity_service(activity_service)
        .build();

    Ok(App::builder()
        .dbus_actor(dbus_actor)
        .activity_actor(activity_actor)
        .config(config)
        .build())
}
