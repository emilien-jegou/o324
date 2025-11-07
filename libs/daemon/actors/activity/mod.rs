use crate::{
    core::supervisor::{Retry, SendableError, StartWithRetry}, // 1. Add `Retry` to imports
    services::activity::ActivityService,
};
use actix::{Actor, Context, Handler, ResponseFuture};
use std::sync::Arc;
use wrap_builder::wrap_builder;

#[wrap_builder(Arc)]
pub struct ActivityActor {
    activity_service: ActivityService,
}

impl Actor for ActivityActor {
    type Context = Context<Self>;
}

impl Handler<Retry<StartWithRetry>> for ActivityActor {
    type Result = ResponseFuture<Result<(), SendableError>>;

    fn handle(&mut self, msg: Retry<StartWithRetry>, _: &mut Context<Self>) -> Self::Result {
        // The original StartWithRetry message is inside the wrapper at `msg.0`
        // We can use it for logging the attempt number.
        let attempt_num = msg.0 .1;
        println!("Handling activity monitoring, attempt #{}", attempt_num);

        let service = self.clone();
        Box::pin(async move {
            match service.activity_service.start_monitoring().await {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error on attempt {}: {:?}", attempt_num, e);
                    Err(Box::new(e) as SendableError)
                }
            }
        })
    }
}
