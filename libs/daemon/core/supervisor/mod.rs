use actix::{dev::ToEnvelope, Actor, Addr, Handler, Message};
use std::error::Error;

mod retry;
pub use retry::{retry_send, Retry, RetryError, RetryStrategy};

// --- Define a Send-able Error Type ---
pub type SendableError = Box<dyn Error + Send + Sync>;

// --- Message Definition ---

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SendableError>")]
#[allow(dead_code)]
pub struct StartWithRetry(pub RetryStrategy, pub u64 /* attempt number */);

// --- Public Helper Function ---

/// Starts and supervises a process for a given actor with a specific retry strategy.
///
/// This function uses a factory to create the actor. If the actor panics, it will
/// be automatically recreated, and the operation will be retried.
///
/// # Arguments
/// * `actor_factory` - A closure that creates and starts the actor, returning its `Addr`.
/// * `strategy` - The `RetryStrategy` to apply.
///
/// # Returns
/// The final result of the operation. This can be:
/// - `Ok(Ok(()))` on success.
/// - `Ok(Err(SendableError))` if the actor returns a permanent error.
/// - `Err(RetryError::Exhausted)` if all retry attempts fail.
pub async fn start_with_retry<A, F>(
    actor_factory: F,
    strategy: RetryStrategy,
) -> Result<Result<(), SendableError>, RetryError>
where
    A: Actor + Handler<Retry<StartWithRetry>> + Send + 'static,
    A::Context: ToEnvelope<A, Retry<StartWithRetry>>,
    F: FnMut() -> Addr<A> + Send + 'static,
{
    let s = strategy.clone();
    let msg_factory = |attempt: u32| StartWithRetry(s.clone(), attempt as u64);

    // Directly call the retry logic without the intermediate Supervisor actor.
    retry_send(actor_factory, msg_factory, strategy).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // A simple worker actor that can be configured to fail or panic.
    struct Worker {
        fail_for: Arc<Mutex<u32>>,
        panic_for: Arc<Mutex<u32>>,
    }

    impl Actor for Worker {
        type Context = actix::Context<Self>;
    }

    impl Handler<Retry<StartWithRetry>> for Worker {
        type Result = actix::ResponseFuture<Result<(), SendableError>>;

        fn handle(&mut self, msg: Retry<StartWithRetry>, _: &mut Self::Context) -> Self::Result {
            let attempt_num = msg.0 .1;
            println!("Worker received attempt #{}", attempt_num);

            // We decide whether to panic in a separate scope to ensure the lock is released.
            let should_panic = {
                let mut panic_for_guard =
                    self.panic_for.lock().expect("Mutex should not be poisoned");
                if *panic_for_guard > 0 {
                    *panic_for_guard -= 1;
                    true // Will panic after the lock is released
                } else {
                    false
                }
            }; // MutexGuard is dropped here, lock is released.

            if should_panic {
                println!("Worker is panicking this attempt.");
                panic!("Simulated panic on attempt {}", attempt_num);
            }

            let mut fail_for = self.fail_for.lock().expect("Mutex should not be poisoned");
            if *fail_for > 0 {
                *fail_for -= 1;
                println!("Worker is failing with an error this attempt.");
                let err: SendableError =
                    format!("Simulated failure on attempt {}", attempt_num).into();
                Box::pin(async { Err(err) })
            } else {
                println!("Worker is succeeding.");
                Box::pin(async { Ok(()) })
            }
        }
    }

    #[actix::test]
    async fn test_start_with_retry_recovers_from_panic_and_succeeds() {
        // Arrange: This worker will panic on attempt 1, fail with an error on
        // attempt 2, and succeed on attempt 3.
        let panic_for = Arc::new(Mutex::new(1));
        let fail_for = Arc::new(Mutex::new(1));

        let worker_factory = move || {
            println!("--- Creating a new Worker instance ---");
            Worker {
                fail_for: fail_for.clone(),
                panic_for: panic_for.clone(),
            }
            .start()
        };

        let strategy = RetryStrategy::Flat {
            max_attempts: Some(5),
            delay: Duration::from_millis(10),
        };

        // Act
        let final_result = start_with_retry(worker_factory, strategy).await;

        // Assert
        assert!(matches!(final_result, Ok(Ok(()))));
    }

    #[actix::test]
    async fn test_start_with_retry_exhausts_after_panics() {
        // Arrange
        let panic_for = Arc::new(Mutex::new(5));
        let fail_for = Arc::new(Mutex::new(0));

        let worker_factory = move || {
            println!("--- Creating a new Worker instance ---");
            Worker {
                fail_for: fail_for.clone(),
                panic_for: panic_for.clone(),
            }
            .start()
        };

        let strategy = RetryStrategy::Flat {
            max_attempts: Some(3),
            delay: Duration::from_millis(10),
        };

        // Act
        let final_result = start_with_retry(worker_factory, strategy).await;

        // Assert
        assert!(matches!(final_result, Err(RetryError::Exhausted)));
    }
}
