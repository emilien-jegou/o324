use o324_core::{Core, StartTaskInput, TaskRef};
use o324_storage::{Task, TaskId, TaskUpdate};
use o324_storage_git::GitStorageConfig;
use tauri::Window;
use tracing::{error, trace};

#[cfg(target_os = "linux")]
mod dbus_plugin;
mod window_emitter;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn list_last_tasks(
    core: tauri::State<'_, Core>,
    count: u64,
) -> std::result::Result<Vec<Task>, String> {
    trace!(count = count, "tauri command - list_last_tasks");
    core.list_last_tasks(count).await.map_err(error_handler)
}

fn error_handler(e: impl std::fmt::Display + std::fmt::Debug) -> String {
    error!("Tauri error during command: {e:?}");
    format!("{}", e)
}

#[tauri::command]
async fn start_new_task(
    core: tauri::State<'_, Core>,
    data: StartTaskInput,
    window: Window,
) -> std::result::Result<(), String> {
    trace!(data = format!("{data:?}"), "tauri command - start_new_task");
    let actions = core.start_new_task(data).await.map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn edit_task(
    core: tauri::State<'_, Core>,
    ulid: TaskId,
    data: TaskUpdate,
    window: Window,
) -> std::result::Result<(), String> {
    trace!(data = format!("{data:?}"), "tauri command - start_new_task");
    let actions = core
        .edit_task(TaskRef::Id(ulid), data)
        .await
        .map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn delete_task_by_ulid(
    core: tauri::State<'_, Core>,
    ulid: String,
    window: Window,
) -> std::result::Result<(), String> {
    trace!(ulid = ulid, "tauri command - delete_task_by_ulid");
    let actions = core.delete_task(ulid).await.map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn stop_current_task(
    core: tauri::State<'_, Core>,
    window: Window,
) -> std::result::Result<(), String> {
    trace!("tauri command - stop_current_task");
    let actions = core.stop_current_task().await.map_err(error_handler)?;
    window_emitter::send_task_action_events(&window, &actions).map_err(error_handler)?;
    Ok(())
}

#[tauri::command]
async fn synchronize_tasks(core: tauri::State<'_, Core>) -> std::result::Result<(), String> {
    trace!("tauri command - synchronize_tasks");
    core.synchronize().await.map_err(error_handler)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_mobile() {
    use o324_config::{Config, CoreConfig, ProfileConfig};

    let config = Config {
        core: CoreConfig {
            computer_name: "android".into(),
            default_profile_name: None,
        },
        profile: sugars::hmap! {
                "default".to_string() => ProfileConfig {
            storage_type: "git".to_string(),
        details: toml::from_str(toml::to_string(&GitStorageConfig {
            connection_name: None,
            git_storage_path: None,
            git_remote_origin_url: "/tmp/aaaaaaa".to_string(),
            git_file_format_type: None,
        }).unwrap().as_str()).unwrap()
                    },
            },
    };

    let choosen_profile: &ProfileConfig = config.profile.get("default").unwrap();

    let storage = o324_storage::load_builtin_storage_from_profile(choosen_profile).unwrap();

    run(Core {
        storage,
        config: config.core,
    })
}

#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_tag("{{app.name}}"),
    );
}

#[cfg(not(target_os = "android"))]
fn init_logging() {
    env_logger::init();
}

pub fn run(core: o324_core::Core) {
    init_logging();
    let mut builder = tauri::Builder::default()
        .manage(core)
        .plugin(tauri_plugin_shell::init());

    #[cfg(target_os = "linux")]
    {
        builder = builder.plugin(dbus_plugin::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            delete_task_by_ulid,
            edit_task,
            list_last_tasks,
            start_new_task,
            stop_current_task,
            synchronize_tasks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
