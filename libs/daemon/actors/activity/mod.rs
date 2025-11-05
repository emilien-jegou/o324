use crate::core::supervisor::{RetryableHandler, WithRetry};
use crate::services::activity::{ActivityService, Error as ActivityError};
use actix::{Context, Message, ResponseFuture}; // We still need some actix imports
use std::sync::Arc;
use wrap_builder::wrap_builder;

#[wrap_builder(Arc)]
pub struct ActivityActor {
    activity_service: ActivityService,
}

// The message remains the same.
#[derive(Message, Clone)]
#[rtype(result = "Result<(), ActivityError>")]
pub struct Serve;

// --- Step 2: Implement `RetryableHandler` instead of `actix::Handler` ---
impl RetryableHandler<Serve> for ActivityActor {
    // The result type is the same as before.
    type Result = ResponseFuture<Result<(), ActivityError>>;

    // The signature changes to match the trait.
    fn handle(
        &mut self,
        _msg: Serve,
        _ctx: &mut Context<WithRetry<Self>>, // We get the wrapper's context
        attempt: u32,                        // We get the current attempt number
    ) -> Self::Result {
        println!("Attempt #{} to start activity monitoring...", attempt);

        // The core async logic is identical.
        // We clone `self` to move it into the async block.
        let service = self.clone();
        Box::pin(async move {
            match service.activity_service.start_monitoring().await {
                Ok(_) => {
                    println!(
                        "Activity monitoring started successfully on attempt #{}",
                        attempt
                    );
                    Ok(())
                }
                Err(e) => {
                    // It's good practice to log the error before returning it.
                    // The error will cause the actor context to stop, triggering a restart.
                    eprintln!("Error on attempt #{}: {:?}. Will retry.", attempt, e);
                    Err(e)
                }
            }
        })
    }
}
