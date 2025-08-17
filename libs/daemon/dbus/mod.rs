use o324_dbus::{define_o324_service_interface_methods, dto};
use std::{error::Error, future::pending, sync::Arc};
use zbus::{connection, fdo, interface};

mod transforms;

use crate::core::{Core, TaskRef};

pub trait O324ServiceInterface {
    define_o324_service_interface_methods!();
}

/// The D-Bus service struct that wraps our application's Core.
pub struct O324Service {
    core: Arc<Core>,
}

#[interface(name = "org.o324.Service1")]
impl O324ServiceInterface for O324Service {
    async fn start_new_task(
        &self,
        input: dto::StartTaskInputDto,
    ) -> fdo::Result<Vec<dto::TaskActionDto>> {
        let core_result = self.core.start_new_task(input.into()).await;
        core_result
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn stop_current_task(&self) -> fdo::Result<Vec<dto::TaskActionDto>> {
        self.core
            .stop_current_task()
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn cancel_current_task(&self) -> fdo::Result<Vec<dto::TaskActionDto>> {
        self.core
            .cancel_current_task()
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn delete_task(&self, task_id: String) -> fdo::Result<Vec<dto::TaskActionDto>> {
        self.core
            .delete_task(task_id)
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn edit_task(
        &self,
        task_ref_str: String,
        update: dto::TaskUpdateDto,
    ) -> fdo::Result<Vec<dto::TaskActionDto>> {
        let task_ref = task_ref_str
            .parse::<TaskRef>() // <-- The fix is here
            .map_err(|e| fdo::Error::InvalidArgs(e.to_string()))?;

        self.core
            .edit_task(task_ref, update.into())
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn list_last_tasks(&self, count: u64) -> fdo::Result<Vec<dto::TaskDto>> {
        self.core
            .list_last_tasks(count)
            .await
            .map(|tasks| tasks.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> fdo::Result<Vec<dto::TaskDto>> {
        self.core
            .list_task_range(start_timestamp, end_timestamp)
            .await
            .map(|tasks| tasks.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

/// The main entry point for the D-Bus service daemon.
pub async fn start_dbus_service(core: Core) -> eyre::Result<()> {
    let service = O324Service {
        core: Arc::new(core),
    };

    // 2. Build and run the D-Bus connection
    let _conn = connection::Builder::session()?
        .name("org.o324.Service")?
        .serve_at("/org/o324/Service", service)?
        .build()
        .await?;

    tracing::info!("D-Bus service running. Waiting for calls.");

    // 3. Keep the service alive
    pending::<()>().await;

    Ok(())
}
