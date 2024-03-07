use fs2::FileExt;
use o324_storage_core::LockType;
use std::fs::File;
use std::io;
use std::path::Path;

pub struct SystemLock {
    file: File,
    lock_type: LockType,
}

impl SystemLock {
    // Attempt to create a new SystemLock, locking the file if possible.
    pub fn try_new(lock_name: &str, lock_type: LockType) -> io::Result<Self> {
        let path = Path::new("/tmp/o324");
        std::fs::create_dir_all(path)?;
        let file = File::create(path.join(format!("lock_{lock_name}")))?;

        Ok(Self { file, lock_type })
    }

    // Attempt to lock the file, returning an error if unable.
    pub fn lock(&self) -> io::Result<()> {
        match self.lock_type {
            LockType::Shared => self.file.lock_shared(),
            LockType::Exclusive => self.file.lock_exclusive(),
        }
    }

    // Unlock the file.
    pub fn unlock(&self) -> io::Result<()> {
        self.file.unlock()
    }
}

#[cfg(test)]
mod tests {
    use nix::unistd::{fork, ForkResult};
    use shared_memory::Shmem;
    use shared_memory::ShmemConf;
    use std::time::Duration;

    use super::*;

    fn append_str_to_shared_memory(shmem: &Shmem, data: &str) -> io::Result<()> {
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
        let lock = SystemLock::try_new("test_0").expect("Failed to create SystemLock");
        lock.lock().expect("Failed to lock");
        lock.unlock().expect("Failed to unlock");
    }

    // TODO: add unix only test flag
    #[test]
    fn test_lock_unlock_multi_process() {
        let shmem = ShmemConf::new()
            .flink(Path::new("/tmp/__sytem_lock_shared_memory_test"))
            .size(1024)
            .force_create_flink()
            .create()
            .unwrap();

        let log = |s: &str| append_str_to_shared_memory(&shmem, &format!("{s}\n\0")).unwrap();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { .. }) => {
                let lock = SystemLock::try_new("test_1").expect("Failed to create SystemLock");
                log("0 : start lock");
                lock.lock().expect("Failed to lock");
                log("0 : locked");
                std::thread::sleep(Duration::from_millis(50));
                log("0 : unlocked");
                // implied unlock -> Drop
            }
            Ok(ForkResult::Child) => {
                std::thread::sleep(Duration::from_millis(20)); // Ensure parent lock first
                let lock = SystemLock::try_new("test_1").expect("Failed to create SystemLock");
                log("1 : start lock");
                lock.lock().expect("Failed to lock");
                log("1 : locked");
                log("1 : unlocked");
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
                "0 : start lock",
                "0 : locked",
                "1 : start lock",
                "0 : unlocked",
                "1 : locked",
                "1 : unlocked",
                "",
            ]
            .join("\n")
        );
    }
}
