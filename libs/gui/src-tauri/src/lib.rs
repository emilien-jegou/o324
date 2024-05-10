use o324_core::{Core, StartTaskInput, TaskRef};
use o324_storage::{Task, TaskId, TaskUpdate};
use tauri::Window;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::{error, trace};

mod dbus_plugin;
mod window_emitter;

fn error_handler(e: impl std::fmt::Display + std::fmt::Debug) -> String {
    error!("Tauri error during command: {e:?}");
    format!("{}", e)
}

struct AppCore(RwLock<Core>);

impl AppCore {
    pub fn new(core: Core) -> Self {
        Self(RwLock::new(core))
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, Core> {
        self.0.read().await
    }

    pub async fn update(&self, new_core: Core) {
        let mut w = self.0.write().await;
        *w = new_core;
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn list_last_tasks(core: tauri::State<'_, AppCore>, count: u64) -> Result<Vec<Task>, String> {
    trace!(count = count, "tauri command - list_last_tasks");
    core.get()
        .await
        .list_last_tasks(count)
        .await
        .map_err(error_handler)
}

#[tauri::command]
async fn start_new_task(
    core: tauri::State<'_, AppCore>,
    data: StartTaskInput,
    window: Window,
) -> Result<(), String> {
    trace!(data = format!("{data:?}"), "tauri command - start_new_task");
    let actions = core
        .get()
        .await
        .start_new_task(data)
        .await
        .map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn get_current_config(
    core: tauri::State<'_, AppCore>,
) -> Result<o324_core::BaseConfig, String> {
    trace!("tauri command - get_current_config");
    let config = core.get().await.get_loaded_config();
    Ok(config)
}

#[tauri::command]
async fn load_profile(
    core: tauri::State<'_, AppCore>,
    app_config: tauri::State<'_, AppConfig>,
    profile: String,
    window: Window,
) -> Result<(), String> {
    trace!("tauri command - load_profile");

    let config = core.get().await.get_loaded_config();
    let new_core =
        o324_core::load_from_config(config, Some(profile.clone())).map_err(error_handler)?;

    // Update the app configuration
    let mut inner = app_config.write().await;
    inner.profile_name = Some(profile);
    std::mem::drop(inner);

    // Load the new profile; require core reload
    core.update(new_core).await;

    // Notify frontend of change, this will reload all tasks
    window_emitter::send_config_reload(&window).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn save_new_config(
    core: tauri::State<'_, AppCore>,
    app_config: tauri::State<'_, AppConfig>,
    config: o324_core::BaseConfig,
    window: Window,
) -> Result<(), String> {
    trace!("tauri command - save_new_config");

    // Extract data from config and drop the lock
    let inner = app_config.read().await;
    let profile_name = inner.profile_name.clone();
    let config_path = inner.config_path.clone();
    std::mem::drop(inner);

    // Load the core storage with the new configuration
    let new_core =
        o324_core::load_from_config(config.clone(), profile_name).map_err(error_handler)?;

    // Try writing the new config to a file
    o324_core::save_config(&config_path, &config).map_err(error_handler)?;

    // Save the new core in the app state
    core.update(new_core).await;

    window_emitter::send_config_reload(&window).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn edit_task(
    core: tauri::State<'_, AppCore>,
    ulid: TaskId,
    data: TaskUpdate,
    window: Window,
) -> std::result::Result<(), String> {
    trace!(data = format!("{data:?}"), "tauri command - start_new_task");
    let actions = core
        .get()
        .await
        .edit_task(TaskRef::Id(ulid), data)
        .await
        .map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn delete_task_by_ulid(
    core: tauri::State<'_, AppCore>,
    ulid: String,
    window: Window,
) -> std::result::Result<(), String> {
    trace!(ulid = ulid, "tauri command - delete_task_by_ulid");
    let actions = core
        .get()
        .await
        .delete_task(ulid)
        .await
        .map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn stop_current_task(
    core: tauri::State<'_, AppCore>,
    window: Window,
) -> std::result::Result<(), String> {
    trace!("tauri command - stop_current_task");
    let actions = core
        .get()
        .await
        .stop_current_task()
        .await
        .map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn synchronize_tasks(core: tauri::State<'_, AppCore>) -> std::result::Result<(), String> {
    trace!("tauri command - synchronize_tasks");
    core.get()
        .await
        .synchronize()
        .await
        .map_err(error_handler)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_mobile() {
    unimplemented!()
}

pub struct AppConfigInner {
    pub profile_name: Option<String>,
    pub config_path: String,
}

pub struct AppConfig(RwLock<AppConfigInner>);

impl AppConfig {
    fn new(inner: AppConfigInner) -> Self {
        Self(RwLock::new(inner))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, AppConfigInner> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, AppConfigInner> {
        self.0.write().await
    }
}

pub fn run(core: o324_core::Core, app_config: AppConfigInner) {
    tauri::Builder::default()
        .manage(AppCore::new(core))
        .manage(AppConfig::new(app_config))
        .plugin(tauri_plugin_shell::init())
        .plugin(dbus_plugin::init())
        .invoke_handler(tauri::generate_handler![
            delete_task_by_ulid,
            edit_task,
            list_last_tasks,
            start_new_task,
            stop_current_task,
            synchronize_tasks,
            get_current_config,
            save_new_config,
            load_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
