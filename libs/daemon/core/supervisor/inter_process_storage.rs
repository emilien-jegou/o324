use bincode::{config, Decode, Encode};
use eyre::Context;
use raw_sync::locks::{LockImpl, LockInit, Mutex};
use shared_memory::{Shmem, ShmemConf, ShmemError};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU8, Ordering};

const SHMEM_SIZE: usize = 65536; // 64 KiB
const INIT_FLAG_SIZE: usize = 8;
const MUTEX_OFFSET: usize = INIT_FLAG_SIZE;
const LENGTH_HEADER_SIZE: usize = std::mem::size_of::<u64>();

pub struct InterProcessStorage<T>
where
    T: Encode + Decode<()> + Default,
{
    _shmem: Shmem,
    mutex_ptr: *mut u8,
    data_ptr: *mut u8,
    _phantom: PhantomData<T>,
}

unsafe impl<T: Encode + Decode<()> + Default + Send> Send for InterProcessStorage<T> {}
unsafe impl<T: Encode + Decode<()> + Default + Send> Sync for InterProcessStorage<T> {}

impl<T> InterProcessStorage<T>
where
    T: Encode + Decode<()> + Default,
{
    /// Creates a new manager for inter-process storage.
    /// This function now handles the creation/opening of the shared memory
    /// and initialization of the mutex.
    pub fn try_new(shmem_name: &str) -> eyre::Result<Self> {
        let shmem = match ShmemConf::new()
            .size(SHMEM_SIZE)
            .os_id(shmem_name)
            .create()
        {
            Ok(m) => m,
            Err(ShmemError::MappingIdExists) => ShmemConf::new().os_id(shmem_name).open()?,
            Err(e) => return Err(e.into()),
        };

        let base_ptr = shmem.as_ptr();
        let init_flag = unsafe { &*(base_ptr as *const AtomicU8) };
        let mutex_ptr = unsafe { base_ptr.add(MUTEX_OFFSET) };
        let data_ptr = unsafe { base_ptr.add(MUTEX_OFFSET + <Mutex>::size_of(None)) };

        if shmem.is_owner() {
            init_flag.store(0, Ordering::Relaxed);
            unsafe { Mutex::new(mutex_ptr, data_ptr) }
                .map_err(|e| eyre::eyre!("Failed to create new mutex: {}", e))?;

            // Get a mutable slice to the data area
            let data_buffer = unsafe {
                let size = SHMEM_SIZE - (MUTEX_OFFSET + <Mutex>::size_of(None));
                std::slice::from_raw_parts_mut(data_ptr, size)
            };
            // Create the byte representation of zero
            let len_bytes = 0u64.to_le_bytes();
            // Copy it into the header portion of the buffer
            data_buffer[..LENGTH_HEADER_SIZE].copy_from_slice(&len_bytes);
            // =========================================================

            // Now, signal that initialization is complete.
            init_flag.store(1, Ordering::SeqCst);
        } else {
            // Wait for the owner to finish the full initialization.
            while init_flag.load(Ordering::SeqCst) != 1 {
                std::thread::yield_now();
            }
        }

        Ok(Self {
            _shmem: shmem,
            mutex_ptr,
            data_ptr,
            _phantom: PhantomData,
        })
    }

    fn get_mutex(&self) -> eyre::Result<Box<dyn LockImpl>> {
        let (mutex, _) = unsafe { Mutex::from_existing(self.mutex_ptr, self.data_ptr) }
            .map_err(|e| eyre::eyre!("Failed to attach to existing mutex: {}", e))?;
        Ok(mutex)
    }

    /// Reads the entire data structure `T` from shared memory.
    pub fn read(&self) -> eyre::Result<T> {
        let mutex = self.get_mutex()?;
        let guard = mutex
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock mutex for reading: {}", e))?;

        let data_ptr: *mut u8 = *guard;
        let data_size = SHMEM_SIZE - (MUTEX_OFFSET + <Mutex>::size_of(None));
        let buffer = unsafe { std::slice::from_raw_parts(data_ptr, data_size) };

        // Step 1: Read the length from the header.
        let len_bytes: [u8; LENGTH_HEADER_SIZE] = buffer[..LENGTH_HEADER_SIZE]
            .try_into()
            .context("Failed to read length header")?;
        let data_len = u64::from_le_bytes(len_bytes) as usize;

        if data_len > 0 {
            // Step 2: Define the slice for the actual payload.
            let payload_start = LENGTH_HEADER_SIZE;
            let payload_end = payload_start + data_len;

            if payload_end > data_size {
                return Err(eyre::eyre!(
                    "Inconsistent length header: length {} exceeds buffer size {}",
                    data_len,
                    data_size
                ));
            }
            let payload_slice = &buffer[payload_start..payload_end];

            // Step 3: Deserialize from the payload slice.
            let (data, _): (T, _) = bincode::decode_from_slice(payload_slice, config::standard())
                .context("Failed to deserialize data from shared memory")?;
            Ok(data)
        } else {
            Ok(T::default())
        }
    }

    /// Performs an atomic write operation on the data in shared memory.
    pub fn write<F>(&self, update_fn: F) -> eyre::Result<()>
    where
        F: FnOnce(&mut T),
    {
        let mutex = self.get_mutex()?;
        let guard = mutex
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock mutex for writing: {}", e))?;

        let data_ptr: *mut u8 = *guard;
        let data_size = SHMEM_SIZE - (MUTEX_OFFSET + <Mutex>::size_of(None));
        let buffer = unsafe { std::slice::from_raw_parts_mut(data_ptr, data_size) };

        // Use split_at_mut to safely get two non-overlapping mutable slices.
        let (header_buffer, payload_buffer) = buffer.split_at_mut(LENGTH_HEADER_SIZE);

        // Step 1: Read current data.
        // We can now safely read from header_buffer...
        let len_bytes: [u8; LENGTH_HEADER_SIZE] = header_buffer
            .try_into()
            .context("Failed to read length from header buffer")?;
        let current_len = u64::from_le_bytes(len_bytes) as usize;

        let mut data: T = if current_len > 0 {
            // ...and use that information to slice payload_buffer.
            let current_payload_slice = &payload_buffer[..current_len];
            bincode::decode_from_slice(current_payload_slice, config::standard())
                .map(|(decoded, _)| decoded)
                .unwrap_or_else(|_| T::default()) // On corruption, start fresh
        } else {
            T::default()
        };

        // Step 2: Apply the user-provided modification.
        update_fn(&mut data);

        // Step 3: Serialize the modified data into the payload buffer.
        let written_bytes = bincode::encode_into_slice(&data, payload_buffer, config::standard())
            .context(
            "Failed to encode data into shared memory slice. Data may be too large.",
        )?;

        // Step 4: Write the new length to the header buffer.
        let new_len_bytes = (written_bytes as u64).to_le_bytes();
        header_buffer.copy_from_slice(&new_len_bytes);

        Ok(())
    }
}
