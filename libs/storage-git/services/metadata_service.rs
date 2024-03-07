use crate::managers::metadata_document_manager::IMetadataDocumentManager;
use o324_storage_core::TaskId;
use shaku::{Component, Interface};
use std::sync::Arc;

pub trait IMetadataService: Interface {
    /// Set id of current running task
    fn set_current_task_reference(&self, task_id: Option<TaskId>) -> eyre::Result<()>;

    /// Get id of current running task
    fn get_current_task_reference(&self) -> eyre::Result<Option<TaskId>>;
}

#[derive(Component)]
#[shaku(interface = IMetadataService)]
pub struct MetadataService {
    #[shaku(inject)]
    metadata_document_manager: Arc<dyn IMetadataDocumentManager>,
}

impl IMetadataService for MetadataService {
    fn set_current_task_reference(&self, task_id: Option<String>) -> eyre::Result<()> {
        self.metadata_document_manager
            .set_current_task_reference(task_id)
    }

    fn get_current_task_reference(&self) -> eyre::Result<Option<String>> {
        self.metadata_document_manager.get_current_task_reference()
    }
}
