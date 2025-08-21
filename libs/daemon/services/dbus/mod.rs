use zbus::connection;

use crate::services::task::TaskService;

pub mod interface;
pub mod transforms;

pub struct DbusService {
    storage_service: TaskService,
}

impl DbusService {
    pub fn new(storage_service: TaskService) -> Self {
        Self { storage_service }
    }
}

impl DbusService {
    pub async fn start_dbus_service(&self) -> eyre::Result<()> {
        let _conn = connection::Builder::session()?
            .name("org.o324.Service")?
            .serve_at(
                "/org/o324/Service",
                interface::O324Service {
                    storage_service: self.storage_service.clone(),
                },
            )?
            .build()
            .await?;

        tracing::info!("D-Bus service running. Waiting for calls.");

        std::future::pending::<()>().await;

        Ok(())
    }
}
