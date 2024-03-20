use dashmap::DashMap;
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    sync::{Arc, Condvar, Mutex},
    thread::ThreadId,
};
use sugars::hset;
use tracing::{instrument, trace};

use super::advisory_lock::SystemLockType;

lazy_static! {
    static ref PROCESS_LOCKS: DashMap<String, Arc<(Mutex<ThreadLockType>, Condvar)>> =
        DashMap::new();
}

enum ThreadLockType {
    Unlocked,
    /// Blocks only during exclusive transactions, allowing concurrent shared transactions,
    /// contains the number of shared watcher
    Shared(HashSet<ThreadId>),
    /// Blocks both during exclusive and shared transactions
    Exclusive(ThreadId),
}

impl ThreadLockType {
    pub fn is_unlocked(&self) -> bool {
        matches!(self, Self::Unlocked)
    }

    pub fn is_exclusive(&self) -> bool {
        matches!(self, Self::Exclusive(_))
    }

    pub fn is_shared(&self) -> bool {
        matches!(self, Self::Shared(_))
    }
}

impl From<SystemLockType> for ThreadLockType {
    fn from(value: SystemLockType) -> Self {
        let thread_id = std::thread::current().id();
        match value {
            SystemLockType::Shared => Self::Shared(hset!(thread_id)),
            SystemLockType::Exclusive => Self::Exclusive(thread_id),
        }
    }
}

impl ThreadLockType {
    fn can_lock(&self, new_lock: &SystemLockType) -> bool {
        match new_lock {
            SystemLockType::Shared => !self.is_exclusive(),
            SystemLockType::Exclusive => {
                if let ThreadLockType::Exclusive(current_thread_id) = self {
                    return current_thread_id == &std::thread::current().id();
                }
                self.is_unlocked()
            }
        }
    }
}

#[derive(Clone)]
pub struct ThreadLock {
    pub name: String,
    lock: Arc<(Mutex<ThreadLockType>, Condvar)>,
}

impl std::fmt::Debug for ThreadLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreadLock[\"{}\"]", self.name)
    }
}

// TODO: all threads locks are exclusive!
impl ThreadLock {
    pub fn new(name: &str) -> Self {
        let lock = PROCESS_LOCKS
            .entry(name.to_owned())
            .or_insert_with(|| Arc::new((Mutex::new(ThreadLockType::Unlocked), Condvar::new())))
            .value()
            .clone();

        ThreadLock {
            name: name.to_owned(),
            lock,
        }
    }

    #[instrument]
    pub fn lock(&self, lock_type: SystemLockType) -> eyre::Result<()> {
        let (lock, cvar) = &*self.lock;
        trace!("Attempting to acquire lock...");
        let mut locked = lock
            .lock()
            .map_err(|_| eyre::eyre!("thread lock error: mutex guard"))?;
        trace!("Mutex acquired, checking peer threads");
        while !locked.can_lock(&lock_type) {
            trace!("Condition not met, waiting");
            locked = cvar
                .wait(locked)
                .map_err(|_| eyre::eyre!("thread lock error: cond guard"))?;
        }
        trace!("Lock successfully acquired");
        *locked = match &*locked {
            ThreadLockType::Unlocked => lock_type.into(),
            ThreadLockType::Shared(set) => {
                let mut new_set = set.clone();
                let thread_id = std::thread::current().id();
                new_set.insert(thread_id);
                ThreadLockType::Shared(new_set)
            }
            ThreadLockType::Exclusive(id) => ThreadLockType::Exclusive(*id),
        };
        Ok(())
    }

    #[instrument]
    pub fn unlock(&self) -> eyre::Result<()> {
        let (lock, cvar) = &*self.lock;
        trace!("Attempting unlocking...");
        let mut locked = lock
            .lock()
            .map_err(|_| eyre::eyre!("thread lock error: mutex guard"))?;
        *locked = match &*locked {
            ThreadLockType::Unlocked => ThreadLockType::Unlocked,
            ThreadLockType::Shared(set) => {
                let mut new_set = set.clone();
                let thread_id = std::thread::current().id();
                new_set.remove(&thread_id);
                match new_set.is_empty() {
                    true => ThreadLockType::Unlocked,
                    false => ThreadLockType::Shared(new_set),
                }
            }
            ThreadLockType::Exclusive(current_thread_id) => {
                let thread_id = std::thread::current().id();
                match thread_id == *current_thread_id {
                    true => ThreadLockType::Unlocked,
                    false => ThreadLockType::Exclusive(*current_thread_id),
                }
            }
        };
        std::mem::drop(locked);
        trace!("Lock released");
        cvar.notify_one();
        trace!("Notified peer threads");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_thread_lock() {
        // Create a new ThreadLock instance
        let lock = ThreadLock::new("test");
        let lock2 = ThreadLock::new("test");

        // Spawn a thread to acquire the lock
        let handle1 = thread::spawn(move || {
            // Attempt to acquire the lock
            let result = lock.lock(SystemLockType::Exclusive);
            assert!(result.is_ok()); // Ensure lock acquisition was successful

            // Simulate some work being done while holding the lock
            // Here, we just sleep for a short duration
            thread::sleep(std::time::Duration::from_secs(1));

            // Unlock the lock
            let result = lock.unlock();
            assert!(result.is_ok()); // Ensure lock release was successful
        });

        // Spawn another thread to acquire the lock concurrently
        let handle2 = thread::spawn(move || {
            // Attempt to acquire the lock
            let result = lock2.lock(SystemLockType::Exclusive);
            assert!(result.is_ok()); // Ensure lock acquisition was successful

            // Simulate some work being done while holding the lock
            // Here, we just sleep for a short duration
            thread::sleep(std::time::Duration::from_secs(1));

            // Unlock the lock
            let result = lock2.unlock();
            assert!(result.is_ok()); // Ensure lock release was successful
        });

        // Wait for both threads to finish
        handle1.join().unwrap();
        handle2.join().unwrap();
    }
}
