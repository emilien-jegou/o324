use o324_storage::StorageBox;

pub mod config;
mod load;
mod utils;

pub use load::{load, load_core};
use ulid::Ulid;

pub struct Core {
    storage: StorageBox,
    /// Ok - found | Err - not found with error reason
    found_config_file: Result<(), eyre::Error>,
}

struct Task {
    id: Ulid,
    start: u64,
    end: Option<u64>,
}

impl Core {
    pub async fn start_task(&self) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;

        self.storage.debug_message();

        //if self.storage.has_active_task().await? == true {
        //return Err(eyre::eyre!(
        //"You cannot have more than one active task at the time"
        //));
        //}

        //self.storage.add_new_task(Task {
        //id: Ulid::new(),
        //start: utils::unix_now(),
        //end: None,
        //});

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
