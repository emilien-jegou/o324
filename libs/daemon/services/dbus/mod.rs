use zbus::connection;

use crate::services::task::TaskService;

pub mod interface;
pub mod transforms;

#[derive(Clone)]
pub struct DbusService {
    task_service: TaskService,
}

impl DbusService {
    pub fn new(task_service: TaskService) -> Self {
        Self { task_service }
    }
}

impl DbusService {
    pub async fn serve(&self) -> eyre::Result<()> {
        let _conn = connection::Builder::session()?
            .name("org.o324.Service")?
            .serve_at(
                "/org/o324/Service",
                interface::O324Service {
                    task_service: self.task_service.clone(),
                },
            )?
            .build()
            .await?;

        tracing::info!("D-Bus service running. Waiting for calls.");

        std::future::pending::<()>().await;

        Ok(())
    }
}
