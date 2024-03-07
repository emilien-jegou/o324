use chrono::{DateTime, Utc};
use lazy_regex::Regex;
use o324_storage_core::{Task, TaskId, TaskUpdate};
use shaku::{Component, Interface};
use std::{collections::HashSet, sync::Arc};
use ulid::Ulid;

use crate::{models::task_document::TaskDocument, module::TaskDocumentStorage, utils};

use super::{config_manager::IConfigManager, file_format_manager::IFileFormatManager};

pub trait ITaskDocumentManager: Interface {
    /// Get the name of the task document
    fn get_task_document_name(&self, ulid: &TaskId) -> eyre::Result<String>;

    /// Get a regex that can match any task document
    fn get_task_document_regex(&self) -> eyre::Result<Regex>;

    fn get_all_tasks(&self) -> eyre::Result<Vec<Task>>;
    fn get_tasks_by_ids(&self, task_ids: &[TaskId]) -> eyre::Result<Vec<Task>>;
    fn create_task(&self, task: Task) -> eyre::Result<()>;
    fn _create_task_batch(&self, task_list: Vec<Task>) -> eyre::Result<()>;
    fn get_task(&self, task_id: &TaskId) -> eyre::Result<Task>;
    fn update_task(&self, task_id: &TaskId, updated_task: TaskUpdate) -> eyre::Result<()>;
    fn delete_task(&self, task_id: &TaskId) -> eyre::Result<()>;
}

#[derive(Component)]
#[shaku(interface = ITaskDocumentManager)]
pub struct TaskDocumentManager {
    #[shaku(inject)]
    file_format_manager: Arc<dyn IFileFormatManager>,
    #[shaku(inject)]
    task_storage: Arc<TaskDocumentStorage>,
    #[shaku(inject)]
    config: Arc<dyn IConfigManager>,
}

impl ITaskDocumentManager for TaskDocumentManager {
    fn get_task_document_name(&self, ulid: &TaskId) -> eyre::Result<String> {
        let date: DateTime<Utc> = Ulid::from_string(ulid)?.datetime().into();
        let formatted_date = date.format("%Y-%m-%d").to_string();
        Ok(formatted_date)
    }

    fn get_task_document_regex(&self) -> eyre::Result<Regex> {
        let rg = self.file_format_manager.file_extension();
        Regex::new(&format!("{}{}$", r"20\d{2}-\d{2}-\d{2}\.", rg)).map_err(From::from)
    }

    fn create_task(&self, task: Task) -> eyre::Result<()> {
        let document_name = self.get_task_document_name(&task.ulid)?;
        let mut data = self
            .task_storage
            .read_as_struct_with_default(&document_name)?;
        data.tasks.insert(task.ulid.clone(), task);
        self.task_storage.write(&document_name, &data)?;
        Ok(())
    }

    fn _create_task_batch(&self, task_list: Vec<Task>) -> eyre::Result<()> {
        use std::collections::HashMap;
        let document_hashmap: HashMap<String, Vec<&Task>> = task_list
            .iter()
            .map(|task| Ok((self.get_task_document_name(&task.ulid)?, task)))
            .collect::<eyre::Result<Vec<(String, &Task)>>>()?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (doc_name, task)| {
                acc.entry(doc_name).or_default().push(task);
                acc
            });

        for (document_name, tasks) in document_hashmap.iter() {
            let mut data = self
                .task_storage
                .read_as_struct_with_default(document_name)?;

            for task in tasks.iter() {
                data.tasks.insert(task.ulid.to_string(), (*task).clone());
            }

            self.task_storage.write(document_name, &data)?;
        }

        Ok(())
    }

    fn get_tasks_by_ids(&self, task_ids: &[TaskId]) -> eyre::Result<Vec<Task>> {
        let task_ids_set: HashSet<String> = task_ids.iter().cloned().collect();

        let document_names = task_ids
            .iter()
            .map(|task_id| self.get_task_document_name(task_id))
            .collect::<eyre::Result<HashSet<String>>>()?;

        let documents = document_names
            .into_iter()
            .map(|path| self.task_storage.read_as_struct_with_default(&path))
            .collect::<eyre::Result<Vec<TaskDocument>>>()?;

        let all_tasks: Vec<Task> = documents
            .into_iter()
            .flat_map(|t| t.tasks.values().cloned().collect::<Vec<Task>>())
            .collect();

        Ok(all_tasks
            .into_iter()
            .filter(|t| task_ids_set.contains(&t.ulid))
            .collect())
    }

    fn get_task(&self, task_id: &TaskId) -> eyre::Result<Task> {
        let document_name = self.get_task_document_name(task_id)?;
        let data = self
            .task_storage
            .read_as_struct_with_default(&document_name)?;

        let task = data
            .tasks
            .get(task_id)
            .ok_or_else(|| eyre::eyre!("Task not found"))?;

        Ok(task.clone())
    }

    fn update_task(&self, task_id: &TaskId, updated_task: TaskUpdate) -> eyre::Result<()> {
        let document_name = self.get_task_document_name(task_id)?;
        let mut data = self
            .task_storage
            .read_as_struct_with_default(&document_name)?;

        let task = data
            .tasks
            .get(task_id)
            .ok_or_else(|| eyre::eyre!("Task not found"))?;

        data.tasks
            .insert(task.ulid.clone(), updated_task.merge_with_task(task));

        self.task_storage.write(&document_name, &data)?;
        Ok(())
    }

    fn delete_task(&self, task_id: &TaskId) -> eyre::Result<()> {
        let document_name = self.get_task_document_name(task_id)?;
        let mut data: TaskDocument = self
            .task_storage
            .read_as_struct_with_default(&document_name)?;

        data.tasks
            .remove(task_id)
            .ok_or_else(|| eyre::eyre!("Task not found"))?;

        self.task_storage.write(&document_name, &data)?;
        Ok(())
    }

    fn get_all_tasks(&self) -> eyre::Result<Vec<Task>> {
        let re = self.get_task_document_regex()?;
        let repository_path = self.config.get_repository_path();

        // Find and parse all task documents
        let all_task_documents = utils::files::find_matching_files(&repository_path, &re)?
            .into_iter()
            .map(|path| self.task_storage.read_as_struct_with_default(&path))
            .collect::<eyre::Result<Vec<TaskDocument>>>()?;

        // Extract and combine every tasks
        let all_tasks = all_task_documents
            .into_iter()
            .flat_map(|v| v.tasks.into_values().collect::<Vec<Task>>())
            .collect::<Vec<Task>>();

        Ok(all_tasks)
    }
}
