use tracing::instrument;

use super::advisory_lock::{AdvisoryLock, SystemLockType};
use super::thread_lock::ThreadLock;

pub struct SystemLock {
    name: String,
    thread_lock: ThreadLock,
    advisory_lock: AdvisoryLock,
}

impl std::fmt::Debug for SystemLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SystemLock[\"{}\"]", self.name)
    }
}

// TODO: advisory lock are tied to the current process, we need a higher level mechanism for same
// process locking; probably through the lock manager
impl SystemLock {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    // Attempt to create a new SystemLock, locking the file if possible.
    pub fn try_new(lock_name: &str) -> eyre::Result<Self> {
        Ok(Self {
            name: lock_name.to_string(),
            thread_lock: ThreadLock::new(lock_name),
            advisory_lock: AdvisoryLock::try_new(lock_name)?,
        })
    }

    // Locks both the thread and system locks.
    #[instrument]
    pub fn lock(&self, lock_type: SystemLockType) -> eyre::Result<()> {
        self.thread_lock.lock(lock_type.clone())?;
        self.advisory_lock.lock(lock_type)?;
        Ok(())
    }

    // Unlocks the system lock. The thread lock is automatically unlocked when the guard is dropped.
    pub fn unlock(&self) -> eyre::Result<()> {
        self.advisory_lock.unlock()?;
        self.thread_lock.unlock()?;
        Ok(())
    }
}

impl Drop for SystemLock {
    #[inline]
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use crate::utils::test_utilities;
    use nix::unistd::{fork, ForkResult};
    use shared_memory::Shmem;
    use shared_memory::ShmemConf;
    use std::time::Duration;
    use tempfile::tempdir;

    use super::*;

    fn append_str_to_shared_memory(shmem: &Shmem, data: &str) -> eyre::Result<()> {
        let max_size = shmem.len();
        let current_content_len = unsafe {
            let data_slice = std::slice::from_raw_parts(shmem.as_ptr(), max_size);
            let current_str = std::str::from_utf8(data_slice).unwrap_or("");

            current_str
                .as_bytes()
                .iter()
                .position(|&x| x == 0)
                .unwrap_or(0)
        };

        let available_space = max_size - current_content_len;
        if data.len() > available_space {
            Err(eyre::eyre!("Not enough space in shared memory"))?;
        }

        unsafe {
            let offset_ptr = shmem.as_ptr().add(current_content_len);
            std::ptr::copy_nonoverlapping(data.as_ptr(), offset_ptr, data.len());
        }

        Ok(())
    }

    #[test]
    fn test_lock_unlock_0() {
        let lock = SystemLock::try_new("test_0").expect("Failed to create SystemLock");
        lock.lock(SystemLockType::Exclusive)
            .expect("Failed to lock");
        lock.unlock().expect("Failed to unlock");
    }

    #[test]
    fn test_lock_unlock_multi_process() {
        let lock_name = test_utilities::random_string(16);
        let tmp = tempdir().unwrap();
        let shmem = ShmemConf::new()
            .flink(tmp.path().join("sytem_lock_shared_memory_test"))
            .size(1024)
            .force_create_flink()
            .create()
            .unwrap();

        let log = |s: &str| append_str_to_shared_memory(&shmem, &format!("{s}, \0")).unwrap();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { .. }) => {
                std::thread::sleep(Duration::from_millis(20)); // Ensure parent lock first
                let lock = SystemLock::try_new(&lock_name).expect("Failed to create SystemLock");
                log("1:start");
                lock.lock(SystemLockType::Exclusive)
                    .expect("Failed to lock");
                log("1:locked");
                log("1:unlocked");
            }
            Ok(ForkResult::Child) => {
                let lock = SystemLock::try_new(&lock_name).expect("Failed to create SystemLock");
                log("0:start");
                lock.lock(SystemLockType::Exclusive)
                    .expect("Failed to lock");
                log("0:locked");
                std::thread::sleep(Duration::from_millis(150));
                log("0:unlocked");
                std::process::exit(0); // Exit to avoid running the rest of the test in the child
            }
            Err(_) => panic!("Fork failed"),
        };

        std::thread::sleep(Duration::from_millis(250));
        let data_slice = unsafe { std::slice::from_raw_parts(shmem.as_ptr(), shmem.len()) };
        let mut s = std::str::from_utf8(data_slice).unwrap().to_string();
        s.truncate(s.trim_end_matches('\0').len());
        assert_eq!(
            s,
            [
                "0:start",
                "0:locked",
                "1:start",
                "0:unlocked",
                "1:locked",
                "1:unlocked",
                "",
            ]
            .join(", ")
        );
    }
}
