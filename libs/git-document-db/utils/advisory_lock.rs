use file_guard::os::raw_file_lock;
use file_guard::Lock;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use tracing::{instrument, trace};

#[derive(Debug, Clone, PartialEq)]
pub enum SystemLockType {
    /// Blocks only during exclusive transactions, allowing concurrent shared transactions
    Shared,
    /// Blocks both during exclusive and shared transactions
    Exclusive,
}

impl From<SystemLockType> for Lock {
    fn from(val: SystemLockType) -> Self {
        match val {
            SystemLockType::Exclusive => Lock::Exclusive,
            SystemLockType::Shared => Lock::Shared,
        }
    }
}

pub struct AdvisoryLock {
    pub name: String,
    file: File,
}

impl std::fmt::Debug for AdvisoryLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdvisoryLock[\"{}\"]", self.name)
    }
}

// TODO: advisory lock are tied to the current process, we need a higher level mechanism for same
// process locking; probably through the lock manager
impl AdvisoryLock {
    // Attempt to create a new SystemLock, locking the file if possible.
    pub fn try_new(lock_name: &str) -> io::Result<Self> {
        let path = Path::new("/tmp/o324-db-lock");
        std::fs::create_dir_all(path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.join(format!("lock_{lock_name}")))?;

        Ok(Self {
            name: lock_name.to_owned(),
            file,
        })
    }

    // Attempt to lock the file, returning an error if unable.
    #[instrument]
    pub fn lock(&self, lock_type: SystemLockType) -> io::Result<()> {
        unsafe {
            trace!("Attempting locking...");
            raw_file_lock(&self.file, Some(lock_type.into()), 0, 1, true)?;
            trace!("Lock successfully acquired");
        };
        Ok(())
    }

    // Unlock the file.
    #[instrument]
    pub fn unlock(&self) -> io::Result<()> {
        unsafe {
            trace!("Attempting unlocking...");
            raw_file_lock(&self.file, None, 0, 1, false)?;
            trace!("Lock released");
        };
        Ok(())
    }
}

impl Drop for AdvisoryLock {
    #[inline]
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

#[cfg(test)]
mod tests {
    use nix::unistd::{fork, ForkResult};
    use std::time::Duration;
    use tempfile::tempdir;

    use crate::utils::system_lock::SystemLock;
    use crate::utils::test_utilities;

    use super::*;

    #[cfg(target_os = "android")]
    fn append_str_to_shared_memory(shmem: &shared_memory::Shmem, data: &str) -> io::Result<()> {
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
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Not enough space in shared memory",
            ));
        }

        unsafe {
            let offset_ptr = shmem.as_ptr().add(current_content_len);
            std::ptr::copy_nonoverlapping(data.as_ptr(), offset_ptr, data.len());
        }

        Ok(())
    }

    #[test]
    fn test_lock_unlock_0() {
        let lock = AdvisoryLock::try_new("test_0").expect("Failed to create SystemLock");
        lock.lock(SystemLockType::Exclusive)
            .expect("Failed to lock");
        lock.unlock().expect("Failed to unlock");
    }

    #[test]
    #[cfg(target_os = "android")]
    fn test_lock_unlock_multi_process() {
        let lock_name = test_utilities::random_string(16);
        let tmp = tempdir().unwrap();
        let shmem = shared_memory::ShmemConf::new()
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

        std::thread::sleep(Duration::from_millis(50));
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
