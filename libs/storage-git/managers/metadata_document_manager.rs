use crate::{models::metadata_document::MetadataDocument, module::MetadataDocumentStorage};
use o324_storage_core::TaskId;
use shaku::{Component, Interface};
use std::{collections::BTreeSet, sync::Arc};

pub trait IMetadataDocumentManager: Interface {
    /// Get the name of the metadata document
    fn get_metadata_document_name(&self) -> String;

    /// Set the current metadata document
    fn set_document(&self, meta: MetadataDocument) -> eyre::Result<()>;

    /// Get the current metadata document
    fn get_document(&self) -> eyre::Result<MetadataDocument>;

    /// Save a reference to a task
    fn save_task_reference(&self, task_id: &str) -> eyre::Result<()>;

    /// Save multiple task reference at once
    #[allow(dead_code)]
    fn save_task_reference_batch(&self, task_ids: &[String]) -> eyre::Result<()>;

    /// Remove a reference to a task
    fn delete_task_reference(&self, task_id: &str) -> eyre::Result<()>;

    /// Set id of current running task
    fn set_current_task_reference(&self, task_id: Option<TaskId>) -> eyre::Result<()>;

    /// Get id of current running task
    fn get_current_task_reference(&self) -> eyre::Result<Option<TaskId>>;

    /// Get the list of all task ids
    fn get_task_reference_list(&self) -> eyre::Result<BTreeSet<TaskId>>;
}

#[derive(Component)]
#[shaku(interface = IMetadataDocumentManager)]
pub struct MetadataDocumentManager {
    #[shaku(inject)]
    metadata_storage: Arc<MetadataDocumentStorage>,
}

impl IMetadataDocumentManager for MetadataDocumentManager {
    fn get_metadata_document_name(&self) -> String {
        "__metadata".to_string()
    }

    fn get_document(&self) -> eyre::Result<MetadataDocument> {
        self.metadata_storage
            .read_as_struct_with_default(&self.get_metadata_document_name())
    }

    fn set_document(&self, meta: MetadataDocument) -> eyre::Result<()> {
        self.metadata_storage
            .write(&self.get_metadata_document_name(), &meta)?;
        Ok(())
    }

    fn save_task_reference(&self, task_id: &str) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        metadata.task_refs.insert(task_id.to_string());
        self.set_document(metadata)?;
        Ok(())
    }

    fn save_task_reference_batch(&self, task_ids: &[String]) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        for task_id in task_ids.iter() {
            metadata.task_refs.insert(task_id.to_string());
        }
        self.set_document(metadata)?;
        Ok(())
    }

    fn delete_task_reference(&self, task_id: &str) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        metadata.task_refs.remove(task_id);
        self.set_document(metadata)?;
        Ok(())
    }

    fn set_current_task_reference(&self, task_id: Option<String>) -> eyre::Result<()> {
        let mut metadata = self.get_document()?;
        metadata.current = task_id;
        self.set_document(metadata)?;
        Ok(())
    }

    fn get_current_task_reference(&self) -> eyre::Result<Option<String>> {
        let metadata = self.get_document()?;
        Ok(metadata.current)
    }

    fn get_task_reference_list(&self) -> eyre::Result<BTreeSet<String>> {
        let metadata = self.get_document()?;
        Ok(metadata.task_refs.iter().cloned().collect())
    }
}
