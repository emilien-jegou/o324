use actix::{dev::ToEnvelope, prelude::*};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RetryError {
    #[error("All retry attempts failed")]
    Exhausted,
}

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Exponential {
        max_attempts: Option<u32>,
        initial_delay: Duration,
        multiplier: f64,
        max_delay: Option<Duration>,
    },
    Flat {
        max_attempts: Option<u32>,
        delay: Duration,
    },
    NoRetry,
}

impl RetryStrategy {
    fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        match self {
            RetryStrategy::NoRetry => {
                if attempt > 1 {
                    None
                } else {
                    Some(Duration::ZERO)
                }
            }
            RetryStrategy::Flat { max_attempts: max_attempts, delay } => {
                if max_attempts.map_or(false, |max| attempt > max) {
                    return None;
                }
                if attempt == 1 {
                    Some(Duration::ZERO)
                } else {
                    Some(*delay)
                }
            }
            RetryStrategy::Exponential {
                max_attempts,
                initial_delay,
                multiplier,
                max_delay,
            } => {
                if max_attempts.map_or(false, |max| attempt > max) {
                    return None;
                }
                if attempt == 1 {
                    return Some(Duration::ZERO);
                }
                let power = attempt as f64 - 2.0;
                let delay_secs = initial_delay.as_secs_f64() * multiplier.powf(power);
                let mut calculated_delay = Duration::from_secs_f64(delay_secs);
                if let Some(max) = max_delay {
                    calculated_delay = calculated_delay.min(*max);
                }
                Some(calculated_delay)
            }
        }
    }
}

/// A wrapper to indicate that a message is being sent as part of a retry operation.
#[derive(Debug)]
pub struct Retry<M: Message>(pub M);

impl<M: Message> Message for Retry<M> {
    type Result = M::Result;
}

/// A trait to determine if a message's result is considered successful.
pub trait IsSuccess {
    fn is_success(&self) -> bool;
}

/// Implementation of `IsSuccess` for any `Result`.
impl<T, E> IsSuccess for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }
}

/// Sends a message to an actor with a retry strategy, using a message factory.
/// It can also recreate the actor using an actor factory if it panics.
pub async fn retry_send<A, M, F, AF>(
    mut actor_factory: AF,
    mut msg_factory: F,
    strategy: RetryStrategy,
) -> Result<M::Result, RetryError>
where
    M: Message + Send + 'static,
    M::Result: Send + IsSuccess,
    A: Actor + Handler<Retry<M>>,
    A::Context: ToEnvelope<A, Retry<M>>,
    F: FnMut(u32) -> M + Send,
    // FIX: We now accept an actor factory that can be called to create new actors.
    AF: FnMut() -> Addr<A> + Send,
{
    let mut attempt = 1;
    // Create the first instance of the actor.
    let mut addr = actor_factory();

    loop {
        let delay = match strategy.delay_for_attempt(attempt) {
            Some(delay) => delay,
            None => return Err(RetryError::Exhausted),
        };

        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }

        let current_msg = msg_factory(attempt);
        let retry_msg = Retry(current_msg);

        match addr.send(retry_msg).await {
            // FIX: A MailboxError now triggers actor recreation. This happens when
            // the actor panics and gets stopped by the Actix runtime.
            Err(_mailbox_error) => {
                tracing::warn!(
                    "Actor mailbox error on attempt {}. Actor may have panicked. Recreating...",
                    attempt
                );
                addr = actor_factory(); // Recreate the actor.
                attempt += 1;
                continue;
            }
            Ok(actor_result) => {
                if actor_result.is_success() {
                    return Ok(actor_result);
                } else {
                    attempt += 1;
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Message, Debug, PartialEq)]
    #[rtype(result = "Result<u32, ()>")]
    struct Ping {
        value: u32,
        attempt: u32,
    }

    struct MyActor {
        fail_for: Arc<Mutex<u32>>,
        panic_for: Arc<Mutex<u32>>,
        total_call_count: Arc<Mutex<u32>>,
        instance_call_count: u32,
    }

    impl Actor for MyActor {
        type Context = Context<Self>;
        fn started(&mut self, _ctx: &mut Self::Context) {
            println!("--- MyActor instance created ---");
        }
    }

    impl Handler<Retry<Ping>> for MyActor {
        type Result = ResponseFuture<Result<u32, ()>>;

        fn handle(&mut self, msg: Retry<Ping>, _: &mut Self::Context) -> Self::Result {
            self.instance_call_count += 1;

            enum Action {
                Panic,
                Fail,
                Succeed,
            }

            // Determine the action to take in a separate scope to release all locks.
            let action = {
                let mut total_calls = self
                    .total_call_count
                    .lock()
                    .expect("Mutex should not be poisoned");
                *total_calls += 1;
                println!(
                    "MyActor received attempt #{}. (Total calls: {}, Instance calls: {})",
                    msg.0.attempt, *total_calls, self.instance_call_count
                );

                let mut panic_for_guard =
                    self.panic_for.lock().expect("Mutex should not be poisoned");
                if *panic_for_guard > 0 {
                    *panic_for_guard -= 1;
                    Action::Panic
                } else {
                    let mut fail_for_guard =
                        self.fail_for.lock().expect("Mutex should not be poisoned");
                    if *fail_for_guard > 0 {
                        *fail_for_guard -= 1;
                        Action::Fail
                    } else {
                        Action::Succeed
                    }
                }
            }; // All MutexGuards are dropped here.

            match action {
                Action::Panic => {
                    panic!("Simulating a panic on attempt {}", msg.0.attempt);
                }
                Action::Fail => Box::pin(async { Err(()) }),
                Action::Succeed => Box::pin(async move { Ok(msg.0.value) }),
            }
        }
    }

    // --- Unit Tests for delay_for_attempt ---

    #[test]
    fn test_delay_for_attempt_exponential() {
        let strategy = RetryStrategy::Exponential {
            max_attempts: Some(5),
            initial_delay: Duration::from_millis(10),
            multiplier: 2.0,
            max_delay: Some(Duration::from_millis(50)),
        };
        assert_eq!(strategy.delay_for_attempt(1), Some(Duration::ZERO));
        assert_eq!(
            strategy.delay_for_attempt(2),
            Some(Duration::from_millis(10))
        ); // 10 * 2^0
        assert_eq!(
            strategy.delay_for_attempt(3),
            Some(Duration::from_millis(20))
        ); // 10 * 2^1
        assert_eq!(
            strategy.delay_for_attempt(4),
            Some(Duration::from_millis(40))
        ); // 10 * 2^2
        assert_eq!(
            strategy.delay_for_attempt(5),
            Some(Duration::from_millis(50))
        ); // 10 * 2^3 = 80, capped at 50
        assert_eq!(strategy.delay_for_attempt(6), None);
    }

    #[test]
    fn test_delay_for_attempt_flat_with_none_attempts() {
        let strategy = RetryStrategy::Flat {
            max_attempts: None,
            delay: Duration::from_millis(10),
        };
        assert_eq!(strategy.delay_for_attempt(1), Some(Duration::ZERO));
        assert_eq!(
            strategy.delay_for_attempt(2),
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            strategy.delay_for_attempt(100),
            Some(Duration::from_millis(10))
        ); // Should not exhaust
    }

    // --- Integration Tests ---

    #[actix::test]
    async fn test_exponential_succeeds_after_retries() {
        let fail_for = Arc::new(Mutex::new(2));
        let call_count = Arc::new(Mutex::new(0));
        let actor_factory = || {
            MyActor {
                fail_for: fail_for.clone(),
                panic_for: Arc::new(Mutex::new(0)),
                total_call_count: call_count.clone(),
                instance_call_count: 0,
            }
            .start()
        };
        let strategy = RetryStrategy::Exponential {
            max_attempts: Some(4),
            initial_delay: Duration::from_millis(10),
            multiplier: 2.0,
            max_delay: None,
        };
        let msg_factory = |attempt| Ping { value: 42, attempt };
        let result = retry_send(actor_factory, msg_factory, strategy).await;
        assert_eq!(result, Ok(Ok(42)));
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[actix::test]
    async fn test_exponential_exhausted() {
        let fail_for = Arc::new(Mutex::new(3));
        let call_count = Arc::new(Mutex::new(0));
        let actor_factory = || {
            MyActor {
                fail_for: fail_for.clone(),
                panic_for: Arc::new(Mutex::new(0)),
                total_call_count: call_count.clone(),
                instance_call_count: 0,
            }
            .start()
        };
        let strategy = RetryStrategy::Exponential {
            max_attempts: Some(3),
            initial_delay: Duration::from_millis(10),
            multiplier: 2.0,
            max_delay: None,
        };
        let msg_factory = |attempt| Ping { value: 42, attempt };
        let result = retry_send(actor_factory, msg_factory, strategy).await;
        assert_eq!(result, Err(RetryError::Exhausted));
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[actix::test]
    async fn test_retry_send_recreates_actor_after_panic() {
        let panic_for = Arc::new(Mutex::new(2));
        let fail_for = Arc::new(Mutex::new(1));
        let call_count = Arc::new(Mutex::new(0));
        let actor_factory = || {
            MyActor {
                fail_for: fail_for.clone(),
                panic_for: panic_for.clone(),
                total_call_count: call_count.clone(),
                instance_call_count: 0,
            }
            .start()
        };
        let strategy = RetryStrategy::Flat {
            max_attempts: Some(5),
            delay: Duration::from_millis(10),
        };
        let msg_factory = |attempt| Ping { value: 99, attempt };
        let result = retry_send(actor_factory, msg_factory, strategy).await;
        assert_eq!(result, Ok(Ok(99)));
        assert_eq!(*call_count.lock().unwrap(), 4);
    }
}
