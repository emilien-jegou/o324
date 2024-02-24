use libc::{
    sem_close, sem_open, sem_post, sem_t, sem_unlink, sem_wait, O_CREAT, O_RDWR, SEM_FAILED,
};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use std::ffi::CString;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Simple wrapper over libc semaphore implementation
pub struct Semaphore {
    sem_ptr: *mut sem_t,
    name: CString,
}

unsafe impl Send for Semaphore {} // Safe to transfer between threads.
unsafe impl Sync for Semaphore {} // Safe to access from multiple threads.

lazy_static::lazy_static! {
    static ref TERMINATED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

impl Semaphore {
    pub fn try_new(semaphore_name: &str) -> eyre::Result<Self> {
        let name = CString::new(semaphore_name)?;
        let sem_ptr = unsafe { sem_open(name.as_ptr(), O_CREAT | O_RDWR, 0o600, 1) };

        if sem_ptr == SEM_FAILED {
            let err = io::Error::last_os_error();
            return Err(eyre::eyre!("Failed to open semaphore: {}", err));
        }

        Self::setup_signal_handler();

        Ok(Self { sem_ptr, name })
    }

    fn setup_signal_handler() {
        let term_clone = TERMINATED.clone();
        std::thread::spawn(move || {
            let mut signals =
                Signals::new(TERM_SIGNALS).expect("Unable to register signal handler");
            for _ in signals.forever() {
                term_clone.store(true, Ordering::SeqCst);
            }
        });
    }

    pub fn try_acquire(&mut self) -> eyre::Result<()> {
        let result = unsafe { sem_wait(self.sem_ptr) };
        if result == -1 {
            Err(eyre::eyre!("{}", io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn release(&mut self) -> eyre::Result<()> {
        let result = unsafe { sem_post(self.sem_ptr) };
        if result == -1 {
            Err(eyre::eyre!("{}", io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn cleanup(&self) -> io::Result<()> {
        unsafe {
            sem_close(self.sem_ptr);
            sem_unlink(self.name.as_ptr());
        }
        Ok(())
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        if TERMINATED.load(Ordering::SeqCst) {
            let _ = self.cleanup();
        }
    }
}
