use o324_storage::Task;
use o324_storage::TaskAction;
use serde::{Deserialize, Serialize};
use zbus::blocking::connection;
use zbus::blocking::Connection;
use zbus::object_server::Interface;
use zbus::proxy;
use zbus::zvariant::{Optional, OwnedValue, Type};

pub use zbus::Error;

#[derive(Deserialize, Serialize, Type, PartialEq, Debug)]
#[zvariant(signature = "s")]
pub enum DbusTaskActionType {
    Upsert,
    Delete,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Type)]
pub struct DbusTaskAction {
    pub action_type: DbusTaskActionType,
    pub action: OwnedValue,
}

#[derive(OwnedValue)]
pub struct DbusTaskActionUpsert {
    pub ulid: String,
    pub task_name: String,
    pub project: Optional<String>,
    pub tags: Vec<String>,
    pub start: u64,
    pub end: Optional<u64>,
    pub __version: u32,
}

#[derive(OwnedValue)]
pub struct DbusTaskActionDelete {
    pub ulid: String,
}

impl TryFrom<TaskAction> for DbusTaskAction {
    type Error = eyre::Error;

    fn try_from(value: TaskAction) -> Result<Self, Self::Error> {
        match value {
            TaskAction::Upsert(task) => Ok(DbusTaskAction::try_new_upsert(DbusTaskActionUpsert {
                ulid: task.ulid,
                task_name: task.task_name,
                project: task.project.into(),
                tags: task.tags,
                start: task.start,
                end: task.end.into(),
                __version: task.__version
            })?),
            TaskAction::Delete(ulid) => Ok(DbusTaskAction::try_new_delete(DbusTaskActionDelete {
                ulid,
            })?),
        }
    }
}

impl TryFrom<DbusTaskAction> for TaskAction {
    type Error = eyre::Error;

    fn try_from(value: DbusTaskAction) -> Result<Self, Self::Error> {
        Ok(match value.action_type {
            DbusTaskActionType::Upsert => {
                let mut action = DbusTaskActionUpsert::try_from(value.action)?;
                TaskAction::Upsert(Task {
                    ulid: action.ulid.clone(),
                    task_name: action.task_name.clone(),
                    project: action.project.take(),
                    tags: action.tags,
                    start: action.start,
                    end: action.end.take(),
                    __version: action.__version,
                })
            }
            DbusTaskActionType::Delete => {
                let action = DbusTaskActionDelete::try_from(value.action)?;
                TaskAction::Delete(action.ulid)
            }
        })
    }
}

impl DbusTaskAction {
    pub fn try_new_upsert(action: DbusTaskActionUpsert) -> eyre::Result<Self> {
        Ok(Self {
            action_type: DbusTaskActionType::Upsert,
            action: action.try_into()?,
        })
    }

    pub fn try_new_delete(action: DbusTaskActionDelete) -> eyre::Result<Self> {
        Ok(Self {
            action_type: DbusTaskActionType::Delete,
            action: action.try_into()?,
        })
    }
}

#[proxy(
    interface = "org.o324.gui",
    default_service = "org.o324.gui",
    default_path = "/org/o324/gui"
)]
trait SharedInterface {
    fn notify_task_change(&mut self, action: DbusTaskAction) -> zbus::Result<String>;
}

pub fn create_server<I: Interface>(interface: I) -> eyre::Result<Connection> {
    Ok(connection::Builder::session()
        .and_then(|b| b.name("org.o324.gui"))
        .and_then(|b| b.serve_at("/org/o324/gui", interface))
        .map_err(|e| eyre::eyre!("Failed to build the D-Bus server: {e}"))?
        .build()?)
}

pub fn create_client() -> eyre::Result<SharedInterfaceProxyBlocking<'static>> {
    let connection = Connection::session()?;
    let proxy = SharedInterfaceProxyBlocking::new(&connection)?;
    Ok(proxy)
}
