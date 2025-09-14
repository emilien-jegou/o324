use actix::{Actor, Context, Handler, Message, ResponseFuture};
use std::sync::Arc;
use wrap_builder::wrap_builder;

use crate::{
    core::supervisor::{RetryHelper, Supervised},
    services::activity::{ActivityService, Error as ActivityError},
};

#[wrap_builder(Arc)]
#[allow(dead_code)]
pub struct ActivityActor {
    activity_service: ActivityService,
}

impl Actor for ActivityActor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), ActivityError>")]
struct Serve;

impl Handler<Serve> for ActivityActor {
    type Result = ResponseFuture<Result<(), ActivityError>>;

    fn handle(&mut self, _: Serve, _: &mut Context<Self>) -> Self::Result {
        let ctx = self.clone();
        Box::pin(async move {
            ctx.activity_service.start_monitoring().await?;
            Ok(())
        })
    }
}

impl Supervised<ActivityActor> for ActivityActor {
    fn build_supervised(&self, _helper: RetryHelper) -> ActivityActor {
        self.clone()
    }
}
