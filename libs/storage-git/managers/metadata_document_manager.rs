use crate::models::metadata_document::MetadataDocument;
use git_document_db::{IQueryRunner, SharedQueryRunner};
use std::collections::BTreeSet;

pub struct MetadataDocumentManager<'a> {
    query_runner: &'a SharedQueryRunner<'a>,
}

impl<'a> MetadataDocumentManager<'a> {
    pub fn load(query_runner: &'a SharedQueryRunner<'a>) -> Self {
        Self { query_runner }
    }

    /// Get the name of the metadata document
    pub fn get_metadata_document_id(&self) -> &'static str {
        "__metadata"
    }

    /// Get the current metadata document
    pub fn get_document(&self) -> eyre::Result<MetadataDocument> {
        let document_id = self.get_metadata_document_id();
        let mut data = self
            .query_runner
            .get::<MetadataDocument>(document_id)?
            .unwrap_or_default();
        data.id = document_id.to_string();
        Ok(data)
    }

    /// Set the current metadata document
    pub fn set_document(&self, meta: MetadataDocument) -> eyre::Result<()> {
        self.query_runner.save(&meta)?;
        Ok(())
    }

    /// Save a reference to a task
    pub fn save_task_reference(&self, task_id: &str) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        metadata.task_refs.insert(task_id.to_string());
        self.set_document(metadata)?;
        Ok(())
    }

    /// Save multiple task reference at once
    pub fn save_task_reference_batch(&self, task_ids: &[String]) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        for task_id in task_ids.iter() {
            metadata.task_refs.insert(task_id.to_string());
        }
        self.set_document(metadata)?;
        Ok(())
    }

    /// Remove a reference to a task
    pub fn delete_task_reference(&self, task_id: &str) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        metadata.task_refs.remove(task_id);
        self.set_document(metadata)?;
        Ok(())
    }

    pub fn set_current_task_reference(&self, task_id: Option<String>) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        metadata.current = task_id;
        self.set_document(metadata)?;
        Ok(())
    }

    pub fn get_current_task_reference(&self) -> eyre::Result<Option<String>> {
        let metadata = self.get_document()?;
        Ok(metadata.current)
    }

    pub fn get_task_reference_list(&self) -> eyre::Result<BTreeSet<String>> {
        let metadata = self.get_document()?;
        Ok(metadata.task_refs.iter().cloned().collect())
    }
}
