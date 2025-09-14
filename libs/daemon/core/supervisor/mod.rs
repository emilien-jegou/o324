use actix::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
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

    /// Calculates the delay for a given attempt number.
    /// Attempt numbers are 1-based.
    /// Returns Some(Duration) for the delay, or None if max attempts are exceeded.
    fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt == 0 {
            return None; // Should not happen with 1-based logic
        }
        match self {
            RetryStrategy::NoRetry => {
                if attempt > 1 {
                    None
                } else {
                    Some(Duration::ZERO)
                }
            }
            RetryStrategy::Flat { attempts, delay } => {
                if let Some(max) = attempts {
                    if attempt > *max {
                        return None;
                    }
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
                if let Some(max) = max_attempts {
                    if attempt > *max {
                        return None;
                    }
                }
                if attempt == 1 {
                    return Some(Duration::ZERO);
                }
                let delay_secs =
                    initial_delay.as_secs_f64() * multiplier.powf((attempt - 2) as f64);
                let mut calculated_delay = Duration::from_secs_f64(delay_secs);
                if let Some(max) = max_delay {
                    if calculated_delay > *max {
                        calculated_delay = *max;
                    }
                }
                Some(calculated_delay)
            }
        }
    }
}

// =================================================================
// SECTION 2: Reusable Retry Abstraction
// =================================================================

/// A helper struct to encapsulate retry state and logic.
/// The user's actor will hold an instance of this.
#[derive(Clone)]
pub struct RetryHelper {
    strategy: RetryStrategy,
    attempt: u32,
}

impl RetryHelper {
    /// This method contains the generic retry logic that can be called
    /// from any actor's `restarting` method.
    pub fn handle_restarting<A>(&self, ctx: &mut <A as Actor>::Context)
    where
        A: Actor<Context = Context<A>>,
    {
        println!(
            "Actor is preparing to restart for attempt #{}...",
            self.attempt
        );

        match self.strategy.delay_for_attempt(self.attempt) {
            Some(delay) => {
                if !delay.is_zero() {
                    println!("...delaying restart by {:?}.", delay);
                    let sleep_fut = tokio::time::sleep(delay).into_actor(unsafe {
                        // This is a bit of a necessary evil to make this generic.
                        // We know `self` in the actor's `restarting` method is valid,
                        // so we can cast the context to satisfy the trait bounds of `into_actor`.
                        // This is safe because the lifetime of the actor (`A`) is managed by the context.
                        std::mem::transmute::<&mut A::Context, &mut A>(ctx)
                    });
                    ctx.wait(sleep_fut);
                } else {
                    println!("...restarting immediately (zero delay).");
                }
            }
            None => {
                println!(
                    "...max attempts ({}) reached. Actor will not restart.",
                    self.strategy.max_attempts_display()
                );
                ctx.stop();
                System::current().stop();
            }
        }
    }
}

// ADDED: The new Supervised trait that acts as a factory.
/// A trait for actors that can be built by the retry supervisor.
/// This acts as a factory.
pub trait Supervised<A>
where
    A: Actor<Context = Context<A>>,
{
    /// Creates a new instance of the supervised actor.
    fn build_supervised(&self, helper: RetryHelper) -> A;
}

// MODIFIED: `start_with_retry` now takes a type implementing our new `Supervised` trait.
/// A factory function to start a supervised actor with a given retry strategy.
///
/// - `strategy`: The `RetryStrategy` to use.
/// - `actor_factory`: An object that implements the `Supervised<A>` trait to build the actor.
pub fn start_with_retry<A, S>(strategy: RetryStrategy, actor: S) -> Addr<A>
where
    A: Actor<Context = Context<A>> + actix::Supervised,
    S: Supervised<A> + Clone + 'static,
{
    let attempt_counter = Rc::new(Cell::new(0_u32));

    Supervisor::start(move |_| {
        let current_attempt = attempt_counter.get() + 1;
        attempt_counter.set(current_attempt);

        let helper = RetryHelper {
            strategy: strategy.clone(),
            attempt: current_attempt,
        };

        actor.build_supervised(helper)
    })
}

// =================================================================
// SECTION 3: Example Actor and Application Logic
// =================================================================

#[derive(Message)]
#[rtype(result = "()")]
struct Die;

/// MyActor is now much simpler. It just needs to hold the helper.
#[derive(Clone)]
struct MyActor {
    pub name: String,
    pub retry_helper: Option<RetryHelper>,
}

impl Supervised<MyActor> for MyActor {
    fn build_supervised(&self, _: RetryHelper) -> MyActor {
        self.clone()
    }
}

impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        // We can access the attempt number for logging if needed
        println!("MyActor started! (Attempt #{})", self.retry_helper.attempt);
    }
}

// MODIFIED: This now implements the real `actix::Supervised` trait, which is required by `actix::Supervisor`.
impl actix::Supervised for MyActor {
    fn restarting(&mut self, ctx: &mut Context<MyActor>) {
        self.retry_helper.handle_restarting::<Self>(ctx);
    }
}

impl Handler<Die> for MyActor {
    type Result = ();

    fn handle(&mut self, _: Die, ctx: &mut Context<MyActor>) {
        println!(
            "MyActor received Die message on attempt #{}, stopping...",
            self.retry_helper.attempt
        );
        ctx.stop();
    }
}

/// Helper function to run a test with a given strategy
fn run_test(strategy: RetryStrategy) {
    println!("\n--- Running test with strategy: {:?} ---", strategy);
    let mut sys = System::new();

    let addr = sys.block_on(async {
        // MODIFIED: We now pass an instance of our factory struct instead of a closure.
        start_with_retry(strategy, MyActor)
    });

    // Send the first Die message to trigger the first restart
    addr.do_send(Die);

    // In a real app, you wouldn't do this, but for demonstration, we'll
    // schedule more "failures" to see the backoff in action.
    let addr_clone = addr.clone();
    actix::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        addr_clone.do_send(Die);
    });

    let addr_clone2 = addr.clone();
    actix::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        addr_clone2.do_send(Die);
    });

    sys.run().unwrap();
    println!("--- Test finished ---");
}

fn main() {
    // 1. Test with a Flat strategy (3 attempts, 1s delay)
    let flat_strategy = RetryStrategy::Flat {
        attempts: Some(3),
        delay: Duration::from_secs(1),
    };

    run_test(flat_strategy);

    // 2. Test with an Exponential strategy
    let exponential_strategy = RetryStrategy::Exponential {
        max_attempts: Some(4),
        initial_delay: Duration::from_millis(500),
        multiplier: 2.0,
        max_delay: Some(Duration::from_secs(3)),
    };

    run_test(exponential_strategy);

    // 3. Test with NoRetry
    let no_retry_strategy = RetryStrategy::NoRetry;
    run_test(no_retry_strategy);
}
