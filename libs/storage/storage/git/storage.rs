use super::{config::GitStorageConfig, transaction::GitTransaction};
use crate::{
    core::task::{TaskId, TaskUpdate},
    utils::{files, time},
    PinFuture, Storage, StorageBox, StorageConfig, Task, TransactionBox,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Default)]
struct DailyDocument {
    tasks: BTreeMap<TaskId, Task>,
}

#[derive(Serialize, Deserialize, Default)]
struct MetadataDocument {
    pub current: Option<String>,
    // Ordered list of tasks by creation date
    pub history: BTreeSet<String>,
}

/// Save data as json inside of a git directory
pub struct GitStorage {
    config: GitStorageConfig,
}

impl GitStorage {
    pub fn new(config: GitStorageConfig) -> Self {
        GitStorage { config }
    }

    async fn check_is_init(&self) -> eyre::Result<()> {
        let storage_path = self.config.get_git_storage_path()?;
        let path = std::path::Path::new(&storage_path);
        files::check_path_is_git_directory(path)
            .map_err(|e| eyre::eyre!("storage is not initialized, got error: {e}"))
    }

    fn get_metadata_path(&self) -> eyre::Result<PathBuf> {
        let storage_path = self.config.get_git_storage_path()?;
        let path = std::path::Path::new(&storage_path);
        let full_path = path.join("__metadata.json");
        Ok(full_path)
    }

    fn get_current_metadata(&self) -> eyre::Result<MetadataDocument> {
        let full_path = self.get_metadata_path()?;
        files::read_json_document_as_struct_with_default(full_path)
    }

    fn set_current_metadata(&self, meta: MetadataDocument) -> eyre::Result<()> {
        let full_path = self.get_metadata_path()?;
        files::save_json_document(full_path, &meta)?;
        Ok(())
    }

    fn get_storage_file_from_ulid(&self, ulid: &TaskId) -> eyre::Result<PathBuf> {
        let storage_path = self.config.get_git_storage_path()?;
        let path = std::path::Path::new(&storage_path);
        let date = time::ulid_to_utc_datetime(ulid)?;
        let formatted_date = date.format("%Y-%m-%d.json").to_string();
        let full_path = path.join(formatted_date);
        Ok(full_path)
    }
}

// - get the path of the storage, default to .local/share/3to4/git-storage-data
// - check if the directory if not, exit, the user has to call ./3to4 init to init the storage with
// the given config
// - try get file for the current day, file have format [yyyy]-[mm]-[dd].json
//    - if doesnt exist -> return false
//    - if exist pass it to git data parser, check if any task as not been ended

impl StorageConfig for GitStorageConfig {
    type Storage = GitStorage;

    fn to_storage(self) -> StorageBox {
        StorageBox::new(GitStorage::new(self))
    }
}

// NOTE: we may only want to expose the try_lock as we want all action to be executed in a
// transaction, methods such as has_active_task may be better suited for the Transaction struct
// instead

impl Storage for GitStorage {
    fn debug_message(&self) {
        println!("Git storage");
        println!("config: {:?}", self.config);
    }

    fn init(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let storage_path = self.config.get_git_storage_path()?;
            let path = std::path::Path::new(&storage_path);
            files::create_dir_if_not_exists_deep(path)?;
            files::init_git_repo_at_path(path)?;
            println!("Initialized git directory on: {storage_path}");
            Ok(())
        })
    }

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move {
            // We want to verify that the repository is a git directory before running the
            // transaction, if it's not, the user will have to run the 'init' command
            self.check_is_init().await?;
            Ok(TransactionBox::new(GitTransaction::try_new()?))
        })
    }

    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let file = self.get_storage_file_from_ulid(&task.ulid)?;

            let mut data: DailyDocument = files::read_json_document_as_struct_with_default(&file)?;

            data.tasks.insert(task.ulid.clone(), task);

            files::save_json_document(&file, &data)?;
            Ok(())
        })
    }

    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>> {
        Box::pin(async move {
            let metadata = self.get_current_metadata()?;
            Ok(metadata.current)
        })
    }

    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let mut metadata = self.get_current_metadata()?;
            metadata.current = task_id;
            self.set_current_metadata(metadata)?;
            Ok(())
        })
    }

    fn get_task(&self, task_id: String) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move {
            let file = self.get_storage_file_from_ulid(&task_id)?;
            let data: DailyDocument = files::read_json_document_as_struct_with_default(&file)?;

            let task = data
                .tasks
                .get(&task_id)
                .ok_or_else(|| eyre::eyre!("Task not found"))?;

            Ok(task.clone())
        })
    }

    fn list_tasks(
        &self,
        _start_timestamp: u64,
        _end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move { todo!() })
    }

    fn update_task(
        &self,
        task_id: String,
        updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let file = self.get_storage_file_from_ulid(&task_id)?;
            let mut data: DailyDocument = files::read_json_document_as_struct_with_default(&file)?;

            let task = data
                .tasks
                .get(&task_id)
                .ok_or_else(|| eyre::eyre!("Task not found"))?;

            data.tasks
                .insert(task.ulid.clone(), updated_task.merge_with_task(task));

            files::save_json_document(&file, &data)?;

            Ok(())
        })
    }

    fn delete_task(&self, task_id: String) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let file = self.get_storage_file_from_ulid(&task_id)?;
            let mut data: DailyDocument = files::read_json_document_as_struct_with_default(&file)?;
            data.tasks
                .remove(&task_id)
                .ok_or_else(|| eyre::eyre!("Task not found"))?;

            files::save_json_document(&file, &data)?;

            Ok(())
        })
    }
}
