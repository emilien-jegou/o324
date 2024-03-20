use o324_dbus_interface::{Error, DbusTaskAction, SharedInterfaceProxyBlocking};
use o324_storage::TaskAction;
use tracing::warn;

/// Notify GUI of changes occuring from the CLI
fn dbus_send_action(
    client: &mut SharedInterfaceProxyBlocking<'_>,
    action: DbusTaskAction,
) -> eyre::Result<()> {
    match client.notify_task_change(action) {
        Err(Error::MethodError(_, _, _)) => {},
        Err(e) => {
            warn!("An error occured during dbus notification: {e}");
        }
        _ => {}
    };

    Ok(())
}

pub fn dbus_notify_task_changes(actions: Vec<TaskAction>) -> eyre::Result<()> {
    let mut client = o324_dbus_interface::create_client()?;

    for task in actions.iter() {
        let dbus_task_action: DbusTaskAction = task.clone().try_into()?;
        dbus_send_action(&mut client, dbus_task_action)?;
    }

    Ok(())
}

