use actix::dev::MessageResponse;
use actix::fut::wrap_future;
use actix::prelude::*;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Exponential {
        max_attempts: Option<u32>,
        initial_delay: Duration,
        multiplier: f64,
        max_delay: Option<Duration>,
    },
    Flat {
        attempts: Option<u32>,
        delay: Duration,
    },
    NoRetry,
}

impl RetryStrategy {
    fn max_attempts_display(&self) -> String {
        match self {
            RetryStrategy::Exponential {
                max_attempts: Some(n),
                ..
            } => n.to_string(),
            RetryStrategy::Flat {
                attempts: Some(n), ..
            } => n.to_string(),
            RetryStrategy::NoRetry => "1".to_string(),
            _ => "unlimited".to_string(),
        }
    }
    // This logic is kept for checking if attempts are exhausted.
    fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt == 0 { return None; }
        match self {
            RetryStrategy::NoRetry => {
                if attempt > 1 { None } else { Some(Duration::ZERO) }
            }
            RetryStrategy::Flat { attempts, delay } => {
                if let Some(max) = attempts {
                    if attempt > *max { return None; }
                }
                if attempt == 1 { Some(Duration::ZERO) } else { Some(*delay) }
            }
            RetryStrategy::Exponential { max_attempts, .. } => {
                if let Some(max) = max_attempts {
                    if attempt > *max { return None; }
                }
                // The actual delay calculation isn't used, but the attempt check is.
                Some(Duration::ZERO)
            }
        }
    }
}

#[derive(Clone)]
pub struct RetryHelper {
    strategy: RetryStrategy,
    pub attempt: u32,
}

impl RetryHelper {
    // CORRECTED: This hook cannot be async. It can only decide if the actor
    // should be stopped permanently. Any delay logic is ignored.
    pub fn handle_restarting<A>(&self, ctx: &mut <A as Actor>::Context)
    where
        A: Actor,
        A::Context: AsyncContext<A>,
    {
        let next_attempt = self.attempt + 1;
        // Check if we have exhausted the number of retries.
        if self.strategy.delay_for_attempt(next_attempt).is_none() {
            // If so, stop the actor for good. The supervisor will not restart it again.
            ctx.stop();
        }
        // Otherwise, do nothing. The supervisor will immediately restart the actor.
    }
}

pub trait RetryableHandler<M: Message>: Clone + Send + Unpin + Sized + 'static {
    type Result: MessageResponse<WithRetry<Self>, M>;
    fn handle(&mut self, msg: M, ctx: &mut Context<WithRetry<Self>>, attempt: u32) -> Self::Result;
}

pub struct WithRetry<S> {
    service: S,
    retry_helper: RetryHelper,
}

impl<S> Actor for WithRetry<S> where S: Clone + Send + Unpin + 'static {
    type Context = Context<Self>;
}

impl<S> Supervised for WithRetry<S> where S: Clone + Send + Unpin + 'static {
    fn restarting(&mut self, ctx: &mut Context<Self>) {
        self.retry_helper.handle_restarting::<Self>(ctx);
    }
}

impl<S, M> Handler<M> for WithRetry<S>
where
    S: RetryableHandler<M>,
    M: Message + Send,
    M::Result: Send,
{
    type Result = <S as RetryableHandler<M>>::Result;
    fn handle(&mut self, msg: M, ctx: &mut Context<Self>) -> Self::Result {
        let current_attempt = self.retry_helper.attempt;
        self.service.handle(msg, ctx, current_attempt)
    }
}

// This factory pattern is correct for actix::Supervisor.
pub fn start_with_retry<S, M>(service: S, strategy: RetryStrategy) -> Addr<WithRetry<S>>
where
    S: RetryableHandler<M>,
    M: Message + Send,
    M::Result: Send,
{
    let attempt_counter = std::rc::Rc::new(std::cell::Cell::new(0));
    Supervisor::start(move |_| {
        attempt_counter.set(attempt_counter.get() + 1);
        WithRetry {
            service: service.clone(),
            retry_helper: RetryHelper {
                strategy: strategy.clone(),
                attempt: attempt_counter.get(),
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::{Actor, Context, Handler, MailboxError, Message, MessageResult};
    use std::time::Duration;
    use tokio::time::timeout;

    #[derive(Message, Clone)]
    #[rtype(result = "Result<String, ()>")]
    struct TestMessage;

    #[derive(Clone)]
    struct AlwaysPanickingService;
    impl RetryableHandler<TestMessage> for AlwaysPanickingService {
        type Result = MessageResult<TestMessage>;
        fn handle(&mut self, _msg: TestMessage, _ctx: &mut Context<WithRetry<Self>>, _attempt: u32) -> Self::Result {
            panic!("This service always fails");
        }
    }

    #[derive(Clone)]
    struct SucceedsOnSecondAttemptService;
    impl RetryableHandler<TestMessage> for SucceedsOnSecondAttemptService {
        type Result = MessageResult<TestMessage>;
        fn handle(&mut self, _msg: TestMessage, _ctx: &mut Context<WithRetry<Self>>, attempt: u32) -> Self::Result {
            if attempt == 1 {
                panic!("Failing on the first attempt");
            }
            MessageResult(Ok("Success".to_string()))
        }
    }

    #[actix_rt::test]
    async fn test_no_retry_stops_after_first_failure() {
        let addr = start_with_retry::<_, TestMessage>(AlwaysPanickingService, RetryStrategy::NoRetry);
        let result = addr.send(TestMessage).await;
        assert!(matches!(result, Err(MailboxError::Closed)));
    }

    #[actix_rt::test]
    async fn test_flat_retry_succeeds_on_second_attempt() {
        let test_body = async {
            // The delay duration is now ignored, but the attempt count is respected.
            let strategy = RetryStrategy::Flat {
                attempts: Some(3),
                delay: Duration::from_millis(20),
            };
            let addr = start_with_retry::<_, TestMessage>(SucceedsOnSecondAttemptService, strategy);

            // First attempt will fail, returning an error because the actor panicked.
            let first_result = addr.send(TestMessage).await;
            assert!(first_result.is_err());

            // Yield control to the scheduler once. This gives the supervisor time
            // to process the panic and immediately restart the actor.
            tokio::task::yield_now().await;

            // The second send goes to the newly created actor instance.
            let second_result = addr.send(TestMessage).await;
            assert_eq!(second_result.unwrap(), Ok("Success".to_string()));
        };
        timeout(Duration::from_secs(1), test_body).await.expect("Test timed out");
    }

    #[actix_rt::test]
    async fn test_flat_retry_exhausts_attempts_and_stops() {
        let test_body = async {
            let max_attempts = 3;
            let strategy = RetryStrategy::Flat {
                attempts: Some(max_attempts),
                delay: Duration::from_millis(20), // Delay is ignored.
            };
            let addr = start_with_retry::<_, TestMessage>(AlwaysPanickingService, strategy);

            for i in 1..=max_attempts {
                let res = addr.send(TestMessage).await;
                assert!(res.is_err(), "Attempt {} should have failed", i);
                // Yield to allow the immediate restart to happen.
                tokio::task::yield_now().await;
            }

            // After 3 failures, the handle_restarting hook will call ctx.stop().
            // The supervisor will not create a 4th actor.
            let final_result = addr.send(TestMessage).await;
            assert!(matches!(final_result, Err(MailboxError::Closed)));
        };
        timeout(Duration::from_secs(1), test_body).await.expect("Test timed out");
    }
}
