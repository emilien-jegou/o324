use std::sync::Arc;
use wrap_builder::wrap_builder;
use zbus::connection;

use crate::services::task_manager::TaskManagerService;

pub mod interface;
pub mod transforms;

#[wrap_builder(Arc)]
pub struct DbusService {
    task_manager_service: TaskManagerService,
}

impl DbusServiceInner {
    pub async fn serve(&self) -> eyre::Result<()> {
        let _conn = connection::Builder::session()?
            .name("org.o324.Service")?
            .serve_at(
                "/org/o324/Service",
                interface::O324Service::builder()
                    .task_manager_service(self.task_manager_service.clone())
                    .build(),
            )?
            .build()
            .await?;

        tracing::info!("D-Bus service running. Waiting for calls.");

        std::future::pending::<()>().await;

        Ok(())
    }
}
