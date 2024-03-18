use std::sync::Arc;

use crate::{module, providers::git_transaction_provider::GitTransaction};

use super::config::GitStorageConfig;
use o324_storage_core::{
    LockType, PinFuture, Storage, StorageBox, StorageConfig, Task, TaskId, TaskUpdate,
    TransactionBox,
};

/// Save data as json inside of a git directory
pub struct GitStorage {
    config: GitStorageConfig,
    module: module::Module,
}

impl GitStorage {
    pub fn try_new(config: GitStorageConfig) -> eyre::Result<Self> {
        let module = module::build_from_config(&config)?;
        Ok(GitStorage { config, module })
    }
}

impl StorageConfig for GitStorageConfig {
    type Storage = GitStorage;

    fn try_into_storage(self) -> eyre::Result<StorageBox> {
        Ok(StorageBox::new(GitStorage::try_new(self)?))
    }
}

impl Storage for GitStorage {
    fn debug_message(&self) {
        println!("Git storage\nconfig: {:?}", self.config);
    }

    fn init(&self, _config: &o324_config::CoreConfig) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            //let git_manager: GitManager = self.module.resolve();
            //git_manager.init_repository()
            Ok(())
        })
    }

    fn try_lock(&self, transaction_type: LockType) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move {
            //let git_manager: &dyn IGitManager = self.module.resolve();
            // We want to verify that the repository is a git directory before running the
            // transaction, if it's not, the user will have to run the 'init' command
            //git_manager.repository_is_initialized()?;
            //let git_manager: Arc<dyn IGitManager> = self.module.resolve();
            let transaction =
                GitTransaction::try_new(self.module.git_service.clone(), transaction_type)?;

            transaction.try_lock()?;

            Ok(TransactionBox::new(Arc::new(transaction)))
        })
    }

    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { self.module.task_service.create_task(task) })
    }

    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>> {
        Box::pin(async move { self.module.metadata_service.get_current_task_reference() })
    }

    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.module
                .metadata_service
                .set_current_task_reference(task_id)?;
            Ok(())
        })
    }

    fn get_task(&self, task_id: TaskId) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move { self.module.task_service.get_task(task_id) })
    }

    fn list_last_tasks(&self, count: u64) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move { self.module.task_service.list_last_tasks(count) })
    }

    fn list_tasks_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            self.module
                .task_service
                .list_tasks_range(start_timestamp, end_timestamp)
        })
    }

    fn update_task(
        &self,
        task_id: String,
        updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { self.module.task_service.update_task(task_id, updated_task) })
    }

    fn delete_task(&self, task_id: TaskId) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move { self.module.task_service.delete_task(task_id) })
    }

    fn synchronize(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.module.storage_sync_service.sync()?;
            Ok(())
        })
    }
}
