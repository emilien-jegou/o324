use o324_storage::StorageBox;

pub mod config;
mod load;

pub use load::{load, load_core};

pub struct Core {
    storage: StorageBox,
    /// Ok - found | Err - not found with error reason
    found_config_file: Result<(), eyre::Error>,
}

impl Core {
    pub async fn start_task(&self) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;

        self.storage.debug_message();

        lock.release().await?;
        Ok(())
    }

    pub fn get_inner_storage(&self) -> &StorageBox {
        &self.storage
    }

    pub fn has_found_config_file(&self) -> &Result<(), eyre::Error> {
        &self.found_config_file
    }
}
