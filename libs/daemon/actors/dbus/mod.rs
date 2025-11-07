use crate::core::supervisor::{Retry, SendableError, StartWithRetry};
use crate::services::{
    activity::ActivityService, storage_bridge::StorageBridgeService, task::TaskService,
};
use actix::{Actor, Context, Handler, ResponseFuture};
use std::sync::Arc;
use wrap_builder::wrap_builder;
use zbus::connection;

pub mod interface;
pub mod transforms;

#[wrap_builder(Arc)]
pub struct DbusActor {
    connection_name: String,
    serve_at: String,
    task_service: TaskService,
    activity_service: ActivityService,
    storage_bridge_service: StorageBridgeService,
}

impl Actor for DbusActor {
    type Context = Context<Self>;
}

impl Handler<Retry<StartWithRetry>> for DbusActor {
    type Result = ResponseFuture<Result<(), SendableError>>;

    fn handle(&mut self, msg: Retry<StartWithRetry>, _: &mut Context<Self>) -> Self::Result {
        let attempt_num = msg.0 .1;
        tracing::info!("Starting DBus service, attempt #{}", attempt_num);

        let connection_name = self.connection_name.clone();
        let serve_at = self.serve_at.clone();
        let ctx = self.clone();

        Box::pin(async move {
            let result: Result<(), SendableError> = async {
                let _conn = connection::Builder::session()
                    .map_err(|e| Box::new(e) as SendableError)?
                    .name(connection_name)
                    .map_err(|e| Box::new(e) as SendableError)?
                    .serve_at(
                        serve_at, // Use the cloned value
                        interface::O324Service::builder()
                            .task_service(ctx.task_service.clone())
                            .activity_service(ctx.activity_service.clone())
                            .storage_bridge_service(ctx.storage_bridge_service.clone())
                            .build(),
                    )
                    .map_err(|e| Box::new(e) as SendableError)?
                    .build()
                    .await
                    .map_err(|e| Box::new(e) as SendableError)?;

                tracing::info!("D-Bus service running. Waiting for calls.");
                std::future::pending::<()>().await;
                Ok(())
            }
            .await;

            // The result is now already the correct type, so we can just return it.
            if let Err(ref e) = result {
                tracing::error!(
                    "Error starting DBus service on attempt {}: {}",
                    attempt_num,
                    e
                );
            }
            result
        })
    }
}
