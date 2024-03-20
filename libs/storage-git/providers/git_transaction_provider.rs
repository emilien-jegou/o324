use git_document_db::IQueryRunner;
use o324_storage_core::{
    PinFuture, StorageClient, StorageTransaction, Task, TaskAction, TaskId, TaskUpdate,
};
use scc::ebr::Guard;
use teloc::Dependency;

use crate::services::{
    git_service::GitService, metadata_service::MetadataService, task_service::TaskService,
};

#[derive(Dependency)]
pub struct GitTransactionService {
    git_service: GitService,
    task_service: TaskService,
    metadata_service: MetadataService,
}

pub struct GitTransactionServiceLoaded<'a> {
    transaction: git_document_db::Transaction<'a>,
    task_service: &'a TaskService,
    metadata_service: &'a MetadataService,
    transaction_is_active: bool,
    action_history: scc::Queue<TaskAction>,
}

impl GitTransactionService {
    pub fn load(&self) -> eyre::Result<GitTransactionServiceLoaded<'_>> {
        let transaction = self.git_service.start_transaction();
        Ok(GitTransactionServiceLoaded {
            transaction,
            metadata_service: &self.metadata_service,
            task_service: &self.task_service,
            transaction_is_active: true,
            action_history: scc::Queue::default(),
        })
    }
}

impl<'a> Drop for GitTransactionServiceLoaded<'a> {
    fn drop(&mut self) {
        self.transaction.abort().unwrap();
    }
}

impl<'a> StorageTransaction for GitTransactionServiceLoaded<'a> {
    fn release(&mut self) -> eyre::Result<Vec<TaskAction>> {
        if self.transaction_is_active {
            self.transaction_is_active = false;
            self.transaction.release()?;
        }
        Ok(self
            .action_history
            .iter(&Guard::new())
            .cloned()
            .collect::<Vec<TaskAction>>())
    }

    fn abort(&mut self) -> eyre::Result<()> {
        if self.transaction_is_active {
            self.transaction_is_active = false;
            self.transaction.abort()?;
        }
        Ok(())
    }
}

impl<'a> StorageClient for GitTransactionServiceLoaded<'a> {
    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.task_service
                .load(&self.transaction.to_shared_runner())
                .create_task(task.clone())?;
            self.action_history.push(TaskAction::Upsert(task));
            Ok(())
        })
    }

    fn get_current_task_id(&self) -> PinFuture<eyre::Result<Option<TaskId>>> {
        Box::pin(async move {
            self.metadata_service
                .load(&self.transaction.to_shared_runner())
                .get_current_task_reference()
        })
    }

    fn set_current_task_id(&self, task_id: Option<TaskId>) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.metadata_service
                .load(&self.transaction.to_shared_runner())
                .set_current_task_reference(task_id)?;
            Ok(())
        })
    }

    fn get_task(&self, task_id: TaskId) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move {
            self.task_service
                .load(&self.transaction.to_shared_runner())
                .get_task(task_id)
        })
    }

    fn list_last_tasks(&self, count: u64) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            self.task_service
                .load(&self.transaction.to_shared_runner())
                .list_last_tasks(count)
        })
    }

    fn list_tasks_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> PinFuture<eyre::Result<Vec<Task>>> {
        Box::pin(async move {
            self.task_service
                .load(&self.transaction.to_shared_runner())
                .list_tasks_range(start_timestamp, end_timestamp)
        })
    }

    fn update_task(
        &self,
        task_id: String,
        updated_task: TaskUpdate,
    ) -> PinFuture<eyre::Result<Task>> {
        Box::pin(async move {
            let task = self
                .task_service
                .load(&self.transaction.to_shared_runner())
                .update_task(task_id, updated_task)?;
            self.action_history.push(TaskAction::Upsert(task.clone()));
            Ok(task)
        })
    }

    fn delete_task(&self, task_id: TaskId) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.task_service
                .load(&self.transaction.to_shared_runner())
                .delete_task(task_id.clone())?;
            self.action_history.push(TaskAction::Delete(task_id));
            Ok(())
        })
    }
}
