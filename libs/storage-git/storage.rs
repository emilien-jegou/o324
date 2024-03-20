use super::config::GitStorageConfig;
use crate::module;
use git_document_db::IQueryRunner;
use o324_storage_core::{
    PinFuture, Storage, StorageClient, StorageConfig, StorageContainer, StorageFn,
    StorageTransaction, Task, TaskAction, TaskId, TaskUpdate,
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

    fn try_into_storage(self) -> eyre::Result<StorageContainer> {
        Ok(StorageContainer::new(GitStorage::try_new(self)?))
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

    fn transaction(
        &self,
        transaction_fn: Box<StorageFn>,
    ) -> PinFuture<eyre::Result<Vec<TaskAction>>> {
        Box::pin(async move {
            let mut transaction = self.module.git_transaction_service.load()?;
            match transaction_fn(&transaction).await {
                Ok(()) => transaction.release(),
                Err(e) => {
                    transaction.abort()?;
                    Err(e)
                }
            }
        })
    }

    fn transaction_2(&self) -> eyre::Result<Box<dyn StorageTransaction + '_>> {
        let transaction = self.module.git_transaction_service.load()?;
        Ok(Box::new(transaction))
    }

    fn synchronize(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.module.storage_sync_service.sync()?;
            Ok(())
        })
    }
}

impl StorageClient for GitStorage {
    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module.task_service.load(&qr).create_task(task)
        })
    }

    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module
                .metadata_service
                .load(&qr)
                .get_current_task_reference()
        })
    }

    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module
                .metadata_service
                .load(&qr)
                .set_current_task_reference(task_id)?;
            Ok(())
        })
    }

    fn get_task(&self, task_id: TaskId) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module.task_service.load(&qr).get_task(task_id)
        })
    }

    fn list_last_tasks(&self, count: u64) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module.task_service.load(&qr).list_last_tasks(count)
        })
    }

    fn list_tasks_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module
                .task_service
                .load(&qr)
                .list_tasks_range(start_timestamp, end_timestamp)
        })
    }

    fn update_task(
        &self,
        task_id: String,
        updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module
                .task_service
                .load(&qr)
                .update_task(task_id, updated_task)
        })
    }

    fn delete_task(&self, task_id: TaskId) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let client = self.module.git_service.get_client();
            let qr = client.to_shared_runner();
            self.module.task_service.load(&qr).delete_task(task_id)
        })
    }
}
