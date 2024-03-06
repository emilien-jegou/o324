use crate::{
    managers::git_manager::IGitManager, managers::metadata_manager::IMetadataManager,
    models::task_document::TaskDocument, module,
    providers::git_transaction_provider::IGitTransaction, ulid_from_timestamp_with_overwrite,
    utils::files,
};

use super::config::GitStorageConfig;
use chrono::{DateTime, Utc};
use o324_storage_core::{
    PinFuture, Storage, StorageBox, StorageConfig, Task, TaskId, TaskUpdate, Transaction,
    TransactionBox,
};
use shaku::{HasComponent, HasProvider};
use std::path::PathBuf;
use std::{
    collections::HashSet,
    ops::Bound::{Excluded, Included},
};
use ulid::Ulid;

/// Save data as json inside of a git directory
pub struct GitStorage {
    config: GitStorageConfig,
    module: module::GitStorageModule,
}

impl GitStorage {
    pub fn try_new(config: GitStorageConfig) -> eyre::Result<Self> {
        let module = module::build_from_config(&config)?;
        Ok(GitStorage { config, module })
    }

    fn list_tasks_by_ids(&self, task_ids: Vec<TaskId>) -> eyre::Result<Vec<Task>> {
        // List of documents where the tasks are contained, remove duplicate
        let files = task_ids
            .iter()
            .map(|task_id| self.get_storage_file_from_ulid(task_id))
            .collect::<eyre::Result<HashSet<PathBuf>>>()?;

        let documents = files
            .into_iter()
            .map(|path| {
                let doc: TaskDocument = files::read_json_document_as_struct_with_default(path)?;
                Ok(doc)
            })
            .collect::<eyre::Result<Vec<TaskDocument>>>()?;

        let task_ids_set: HashSet<String> = task_ids.into_iter().collect();

        // Combine all tasks object from document and filter them out using boundaries
        let tasks = documents
            .into_iter()
            .map(|doc| doc.tasks)
            .fold(Vec::<(String, Task)>::new(), |mut acc, tasks| {
                acc.extend(tasks);
                acc
            })
            .into_iter()
            .filter(|(id, _)| task_ids_set.contains(id))
            .map(|(_, task)| task)
            .collect::<Vec<Task>>();

        Ok(tasks)
    }

    fn get_storage_file_from_ulid(&self, ulid: &TaskId) -> eyre::Result<PathBuf> {
        let storage_path = self.config.get_git_storage_path()?;
        let path = std::path::Path::new(&storage_path);
        let date: DateTime<Utc> = Ulid::from_string(ulid)?.datetime().into();
        let formatted_date = date.format("%Y-%m-%d.json").to_string();
        let full_path = path.join(formatted_date);
        Ok(full_path)
    }
}

// - get the path of the storage, default to .local/share/o324/git-storage-data
// - check if the directory if not, exit, the user has to call ./o324 init to init the storage with
// the given config
// - try get file for the current day, file have format [yyyy]-[mm]-[dd].json
//    - if doesnt exist -> return false
//    - if exist pass it to git data parser, check if any task as not been ended

impl StorageConfig for GitStorageConfig {
    type Storage = GitStorage;

    fn try_into_storage(self) -> eyre::Result<StorageBox> {
        Ok(StorageBox::new(GitStorage::try_new(self)?))
    }
}

// NOTE: we may want to move all read and write method to the transaction struct instead to
// prevent potential misuse.
impl Storage for GitStorage {
    fn debug_message(&self) {
        println!("Git storage\nconfig: {:?}", self.config);
    }

    fn init(&self, _config: &o324_config::CoreConfig) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let git_manager: &dyn IGitManager = self.module.resolve_ref();
            git_manager.init_repository()
        })
    }

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move {
            let git_manager: &dyn IGitManager = self.module.resolve_ref();
            // We want to verify that the repository is a git directory before running the
            // transaction, if it's not, the user will have to run the 'init' command
            git_manager.repository_is_initialized()?;
            let transaction: Box<dyn IGitTransaction> = self
                .module
                .provide()
                .map_err(|e| eyre::eyre!("couldn't provide transaction: {e}"))?;

            Ok(TransactionBox::new(transaction as Box<dyn Transaction>))
        })
    }

    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let metadata_manager: &dyn IMetadataManager = self.module.resolve_ref();
            let file = self.get_storage_file_from_ulid(&task.ulid)?;
            metadata_manager.save_task_ref(&task.ulid)?;
            let mut data: TaskDocument = files::read_json_document_as_struct_with_default(&file)?;

            data.tasks.insert(task.ulid.clone(), task);

            files::save_json_document(&file, &data)?;
            Ok(())
        })
    }

    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>> {
        Box::pin(async move {
            let metadata_manager: &dyn IMetadataManager = self.module.resolve_ref();
            let metadata = metadata_manager.get_current()?;
            Ok(metadata.current)
        })
    }

    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let metadata_manager: &dyn IMetadataManager = self.module.resolve_ref();
            metadata_manager.set_current_task(task_id)?;
            Ok(())
        })
    }

    fn get_task(&self, task_id: String) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move {
            let file = self.get_storage_file_from_ulid(&task_id)?;
            let data: TaskDocument = files::read_json_document_as_struct_with_default(file)?;

            let task = data
                .tasks
                .get(&task_id)
                .ok_or_else(|| eyre::eyre!("Task not found"))?;

            Ok(task.clone())
        })
    }

    fn list_last_tasks(&self, count: u64) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            let metadata_manager: &dyn IMetadataManager = self.module.resolve_ref();
            let metadata = metadata_manager.get_current()?;
            let task_ids: Vec<String> = metadata
                .task_refs
                .iter()
                .rev()
                .take(count as usize)
                .cloned()
                .collect();

            let tasks = self.list_tasks_by_ids(task_ids)?;
            Ok(tasks)
        })
    }

    fn list_tasks_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            let metadata_manager: &dyn IMetadataManager = self.module.resolve_ref();
            let metadata = metadata_manager.get_current()?;

            // We convert the timestamp to ulid to simplify the search and set the second part to
            // respectively the lowest and highest characters of Crockford 32 to ensure all ulid
            // between the range are found
            let start = ulid_from_timestamp_with_overwrite(start_timestamp, '0')?;
            let end = ulid_from_timestamp_with_overwrite(end_timestamp, 'Z')?;

            // List of task we desire to return
            let task_ids: Vec<String> = metadata
                .task_refs
                .range((Included(start.clone()), Excluded(end.clone())))
                .cloned()
                .collect();

            let tasks = self.list_tasks_by_ids(task_ids)?;
            Ok(tasks)
        })
    }

    fn update_task(
        &self,
        task_id: String,
        updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let file = self.get_storage_file_from_ulid(&task_id)?;
            let mut data: TaskDocument = files::read_json_document_as_struct_with_default(&file)?;

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
            let metadata_manager: &dyn IMetadataManager = self.module.resolve_ref();
            let file = self.get_storage_file_from_ulid(&task_id)?;
            let mut data: TaskDocument = files::read_json_document_as_struct_with_default(&file)?;
            data.tasks
                .remove(&task_id)
                .ok_or_else(|| eyre::eyre!("Task not found"))?;

            files::save_json_document(&file, &data)?;
            metadata_manager.delete_task_ref(&task_id)?;
            Ok(())
        })
    }

    fn synchronize(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let git_manager: &dyn IGitManager = self.module.resolve_ref();
            git_manager.sync()?;
            Ok(())
        })
    }
}
