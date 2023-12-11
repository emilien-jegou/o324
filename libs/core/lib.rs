use o324_storage::{StorageBox, Task};

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

pub struct StartTaskInput {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
}

impl Core {
    pub async fn initialize(&self) -> eyre::Result<()> {
        self.storage.init().await?;
        Ok(())
    }

    pub async fn start_task(&self, input: StartTaskInput) -> eyre::Result<()> {
        let mut lock = self.storage.try_lock().await?;

        if self.storage.has_active_task().await? == true {
            return Err(eyre::eyre!(
                "You cannot have more than one active task at the time"
            ));
        }

        let task_id = Ulid::new().to_string();
        let start_timestamp = utils::unix_now();
        self.storage
            .start_new_task(Task {
                id: task_id,
                task_name: input.task_name,
                project: input.project,
                tags: input.tags,
                start: start_timestamp,
                end: None,
            })
            .await?;

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
