use crate::managers::metadata_document_manager::MetadataDocumentManager;
use crate::managers::task_document_manager::TaskDocumentManager;
use crate::utils;
use o324_storage_core::{Task, TaskUpdate};
use std::ops::Bound::{Excluded, Included};
use teloc::Dependency;

#[derive(Dependency)]
pub struct TaskService {
    metadata_document_manager: MetadataDocumentManager,
    task_document_manager: TaskDocumentManager,
}

impl TaskService {
    pub fn create_task(&self, task: Task) -> eyre::Result<()> {
        self.metadata_document_manager
            .save_task_reference(&task.ulid)?;
        self.task_document_manager.create_task(task)?;
        Ok(())
    }

    pub fn get_task(&self, task_id: String) -> eyre::Result<Task> {
        self.task_document_manager.get_task(&task_id)
    }

    pub fn list_last_tasks(&self, count: u64) -> eyre::Result<Vec<Task>> {
        let task_ids: Vec<String> = self
            .metadata_document_manager
            .get_task_reference_list()?
            .iter()
            .rev()
            .take(count as usize)
            .cloned()
            .collect();

        let tasks = self.task_document_manager.get_tasks_by_ids(&task_ids)?;
        Ok(tasks)
    }

    pub fn list_tasks_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> eyre::Result<Vec<Task>> {
        // We convert the timestamp to ulid to simplify the search and set the second part to
        // respectively the lowest and highest characters of Crockford 32 to ensure all ulid
        // between the range are found
        let start = utils::ulid::ulid_from_timestamp_with_overwrite(start_timestamp, '0')?;
        let end = utils::ulid::ulid_from_timestamp_with_overwrite(end_timestamp, 'Z')?;

        // List of task we desire to return
        let task_ids: Vec<String> = self
            .metadata_document_manager
            .get_task_reference_list()?
            .range((Included(start.clone()), Excluded(end.clone())))
            .cloned()
            .collect();

        let tasks = self.task_document_manager.get_tasks_by_ids(&task_ids)?;
        Ok(tasks)
    }

    pub fn update_task(&self, task_id: String, updated_task: TaskUpdate) -> eyre::Result<()> {
        self.task_document_manager
            .update_task(&task_id, updated_task)?;
        Ok(())
    }

    pub fn delete_task(&self, task_id: String) -> eyre::Result<()> {
        self.task_document_manager.delete_task(&task_id)?;
        self.metadata_document_manager
            .delete_task_reference(&task_id)?;
        Ok(())
    }
}
