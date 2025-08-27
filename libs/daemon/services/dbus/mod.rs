use std::sync::Arc;
use wrap_builder::wrap_builder;
use zbus::connection;

use crate::services::storage_bridge::StorageBridgeService;

use super::task::TaskService;

pub mod interface;
pub mod transforms;

#[wrap_builder(Arc)]
pub struct DbusService {
    task_service: TaskService,
    storage_bridge_service: StorageBridgeService,
}

impl DbusServiceInner {
    pub async fn serve(&self) -> eyre::Result<()> {
        let _conn = connection::Builder::session()?
            .name("org.o324.Service")?
            .serve_at(
                "/org/o324/Service",
                interface::O324Service::builder()
                    .task_service(self.task_service.clone())
                    .storage_bridge_service(self.storage_bridge_service.clone())
                    .build(),
            )?
            .build()
            .await?;

        tracing::info!("D-Bus service running. Waiting for calls.");

        std::future::pending::<()>().await;

        Ok(())
    }
}
