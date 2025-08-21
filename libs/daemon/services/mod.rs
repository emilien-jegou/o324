mod dbus;
mod task;

use crate::{config::Config, core::storage::Storage, entities::MODELS};

#[allow(dead_code)]
pub struct AppState {
    pub storage_service: task::TaskService,
    pub dbus_service: dbus::DbusService,
    pub config: Config,
}

pub fn create_storage_from_config(config: &Config) -> eyre::Result<Storage> {
    let profile_config = config.get_current_profile()?;

    let mut db_path = profile_config.get_storage_location().clone();
    std::fs::create_dir_all(&db_path)?;
    db_path.push("storage.db");
    let storage = Storage::try_new(&db_path, &MODELS)
        .map_err(|e| eyre::eyre!("Couldn't initialize storage on path {db_path:?}: {e}"))?;
    Ok(storage)
}

pub fn build(config: Config) -> eyre::Result<AppState> {
    let storage = create_storage_from_config(&config)?;
    let storage_service = task::TaskService::try_new(storage, config.clone())?;

    let dbus_service = dbus::DbusService::new(storage_service.clone());

    Ok(AppState {
        storage_service,
        dbus_service,
        config,
    })
}
