use std::sync::Arc;
use wrap_builder::wrap_builder;
use zbus::connection;

use crate::services::{
    activity::ActivityService, storage_bridge::StorageBridgeService, task::TaskService,
};

pub mod interface;
pub mod transforms;

#[wrap_builder(Arc)]
pub struct DbusActor {
    task_service: TaskService,
    activity_service: ActivityService,
    storage_bridge_service: StorageBridgeService,
}

use actix::{Actor, Context, Handler, Message, ResponseFuture};

impl Actor for DbusActor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "eyre::Result<()>")]
struct Serve;

impl Handler<Serve> for DbusActor {
    type Result = ResponseFuture<eyre::Result<()>>;

    fn handle(&mut self, _: Serve, _: &mut Context<Self>) -> Self::Result {
        let ctx = self.clone();
        Box::pin(async move {
            let _conn = connection::Builder::session()?
                .name("org.o324.Service")?
                .serve_at(
                    "/org/o324/Service",
                    interface::O324Service::builder()
                        // Use the cloned services here.
                        .task_service(ctx.task_service.clone())
                        .activity_service(ctx.activity_service.clone())
                        .storage_bridge_service(ctx.storage_bridge_service.clone())
                        .build(),
                )?
                .build()
                .await?;

            tracing::info!("D-Bus service running. Waiting for calls.");
            std::future::pending::<()>().await;
            Ok(())
        })
    }
}
