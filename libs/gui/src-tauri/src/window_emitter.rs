use o324_storage::TaskAction;
use tauri::{Manager, Runtime};

pub fn send_task_action_event<R: Runtime>(
    manager: &impl Manager<R>,
    action: &TaskAction,
) -> eyre::Result<()> {
    manager.emit("task_action", action)?;
    Ok(())
}

pub fn send_task_action_events<R: Runtime>(
    manager: &impl Manager<R>,
    actions: &[TaskAction],
) -> eyre::Result<()> {
    for action in actions.iter() {
        send_task_action_event(manager, action)?;
    }
    Ok(())
}

pub fn send_config_reload<R: Runtime>(manager: &impl Manager<R>) -> eyre::Result<()> {
    manager.emit("config_reload", true)?;
    Ok(())
}
