use crate::models::task_document::TaskDocument;
use chrono::{DateTime, Utc};
use git_document_db::{SharedQueryRunner, StoreResult};
use lazy_regex::Regex;
use o324_storage_core::{Task, TaskId, TaskUpdate};
use std::collections::HashSet;
use ulid::Ulid;

pub struct TaskDocumentManager<'a> {
    query_runner: &'a SharedQueryRunner<'a>,
}

impl<'a> TaskDocumentManager<'a> {
    pub fn load(query_runner: &'a SharedQueryRunner<'a>) -> Self {
        Self { query_runner }
    }

    pub fn get_task_document_id(&self, ulid: &TaskId) -> eyre::Result<String> {
        let date: DateTime<Utc> = Ulid::from_string(ulid)?.datetime().into();
        let formatted_date = date.format("%Y-%m-%d").to_string();
        Ok(formatted_date)
    }

    pub fn get_task_document_regex() -> eyre::Result<Regex> {
        Regex::new(r"20\d{2}-\d{2}-\d{2}").map_err(From::from)
    }

    pub fn create_task(&self, task: Task) -> eyre::Result<()> {
        let document_id = self.get_task_document_id(&task.ulid)?;
        let mut data = self
            .query_runner
            .get::<TaskDocument>(&document_id)?
            .unwrap_or_default();
        data.tasks.insert(task.ulid.clone(), task);
        data.id = document_id;
        self.query_runner.save(&data)?;
        Ok(())
    }

    pub fn _create_task_batch(&self, task_list: Vec<Task>) -> eyre::Result<()> {
        use std::collections::HashMap;
        let document_hashmap: HashMap<String, Vec<&Task>> = task_list
            .iter()
            .map(|task| Ok((self.get_task_document_id(&task.ulid)?, task)))
            .collect::<eyre::Result<Vec<(String, &Task)>>>()?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (doc_name, task)| {
                acc.entry(doc_name).or_default().push(task);
                acc
            });

        for (document_name, tasks) in document_hashmap.iter() {
            let mut data: TaskDocument = self
                .query_runner
                .get(document_name)?
                .unwrap_or_else(Default::default);

            for task in tasks.iter() {
                data.tasks.insert(task.ulid.to_string(), (*task).clone());
            }

            self.query_runner.save(&data)?;
        }

        Ok(())
    }

    pub fn get_tasks_by_ids(&self, task_ids: &[TaskId]) -> eyre::Result<Vec<Task>> {
        let task_ids_set: HashSet<String> = task_ids.iter().cloned().collect();

        let document_names = task_ids
            .iter()
            .map(|task_id| self.get_task_document_id(task_id))
            .collect::<eyre::Result<HashSet<String>>>()?;

        let documents = document_names
            .into_iter()
            .map(|path| {
                self.query_runner
                    .get::<TaskDocument>(&path)
                    .map(|d| d.unwrap_or_default())
            })
            .collect::<StoreResult<Vec<TaskDocument>>>()?;

        let all_tasks: Vec<Task> = documents
            .into_iter()
            .flat_map(|t| t.tasks.values().cloned().collect::<Vec<Task>>())
            .collect();

        Ok(all_tasks
            .into_iter()
            .filter(|t| task_ids_set.contains(&t.ulid))
            .collect())
    }

    pub fn get_task(&self, task_id: &TaskId) -> eyre::Result<Task> {
        let document_name = self.get_task_document_id(task_id)?;
        let data = self
            .query_runner
            .get::<TaskDocument>(&document_name)?
            .unwrap_or_default();

        let task = data
            .tasks
            .get(task_id)
            .ok_or_else(|| eyre::eyre!("Task not found"))?;

        Ok(task.clone())
    }

    pub fn update_task(&self, task_id: &TaskId, updated_task: TaskUpdate) -> eyre::Result<Task> {
        let document_name = self.get_task_document_id(task_id)?;
        let mut data = self
            .query_runner
            .get::<TaskDocument>(&document_name)?
            .unwrap_or_default();

        let task = data
            .tasks
            .get(task_id)
            .ok_or_else(|| eyre::eyre!("Task not found"))?;

        let merged = updated_task.merge_with_task(task);
        data.tasks.insert(task.ulid.clone(), merged.clone());

        self.query_runner.save(&data)?;

        Ok(merged)
    }

    pub fn delete_task(&self, task_id: &TaskId) -> eyre::Result<()> {
        let document_name = self.get_task_document_id(task_id)?;
        let mut data = self
            .query_runner
            .get::<TaskDocument>(&document_name)?
            .unwrap_or_default();

        data.tasks
            .remove(task_id)
            .ok_or_else(|| eyre::eyre!("Task not found"))?;

        self.query_runner.save(&data)?;
        Ok(())
    }

    pub fn get_all_tasks(&self) -> eyre::Result<Vec<Task>> {
        let re = Self::get_task_document_regex()?;
        let all_task_documents: Vec<TaskDocument> = self.query_runner.find_matching(&re)?;

        // Extract and combine every tasks
        let all_tasks = all_task_documents
            .into_iter()
            .flat_map(|v| v.tasks.into_values().collect::<Vec<Task>>())
            .collect::<Vec<Task>>();

        Ok(all_tasks)
    }
}
