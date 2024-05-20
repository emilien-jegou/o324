use std::sync::Arc;
use tray::SystemAppVisibility;

mod commands;
mod dbus_plugin;
mod event_emitter;
mod tray;
mod window_emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_mobile() {
    unimplemented!()
}

pub mod state {
    mod config;
    mod core;
    mod notifier;

    pub use config::{AppConfig, AppConfigInner};
    pub use core::AppCore;
    pub use notifier::AppNotifier;
}

pub fn run(core: o324_core::Core, app_config: state::AppConfigInner) {
    let notifier_state = Arc::new(state::AppNotifier::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .on_window_event({
            let notifier_state = notifier_state.clone();
            move |_w, event| match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    notifier_state
                        .app_visibility_emitter
                        .notify(&SystemAppVisibility::Hide);
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .manage(state::AppCore::new(core))
        .manage(state::AppConfig::new(app_config))
        .manage(notifier_state.clone())
        .plugin(tauri_plugin_shell::init())
        .plugin(dbus_plugin::init())
        .setup({
            let system_tray_listener = notifier_state.app_visibility_emitter.subscribe();
            let app_icon_listener = notifier_state.app_icon_emitter.subscribe();

            move |app| {
                let tray = tray::AppTray::setup(app)?;

                system_tray_listener.start_listen({
                    let app_visibility_item = tray.app_visibility_item.clone();
                    move |data| {
                        app_visibility_item.set_app_visibility(match data {
                            SystemAppVisibility::Show => false,
                            SystemAppVisibility::Hide => true,
                        })?;
                        Ok(())
                    }
                })?;

                app_icon_listener.start_listen(move |variant| {
                    tray.set_app_icon_variant(variant)?;
                    Ok(())
                })?;

                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::delete_task_by_ulid,
            commands::edit_task,
            commands::list_last_tasks,
            commands::start_new_task,
            commands::stop_current_task,
            commands::synchronize_tasks,
            commands::get_current_config,
            commands::save_new_config,
            commands::load_profile,
            commands::update_tray_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
