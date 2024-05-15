use crate::window_emitter;
use o324_dbus_interface::DbusTaskAction;
use o324_storage::TaskAction;
use std::future::pending;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Runtime,
};
use tracing::trace;

struct DbusInterface<R: Runtime> {
    app_handle: AppHandle<R>,
}

#[zbus::interface(name = "org.o324.gui")]
impl<R: Runtime> DbusInterface<R> {
    /// Refresh the frontend with updated task without diffing the task store
    /// Mainly used for propagating CLI updates to the GUI on linux
    fn notify_task_change(&mut self, action: DbusTaskAction) -> String {
        trace!("Received task change action: {action:?}");
        action
            .try_into()
            .map(|task_action: TaskAction| {
                window_emitter::send_task_action_event(&self.app_handle, &task_action)
            })
            .map(|e| {
                e.map(|_| "success".to_string())
                    .unwrap_or_else(|e| format!("Frontend is out of reach: {e}"))
            })
            .unwrap_or_else(|e| format!("Received error during notify: {e}"))
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("dbus")
        .setup(move |app_handle: &AppHandle<R>, _| {
            let app = app_handle.clone();
            tokio::task::spawn(async move {
                let entry = DbusInterface { app_handle: app };
                let _conn = o324_dbus_interface::create_server(entry).unwrap();
                trace!("dbus server is now running");
                // Dbus server is now running...
                pending::<()>().await;
            });
            Ok(())
        })
        .build()
}
