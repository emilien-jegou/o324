use crate::managers::metadata_document_manager::MetadataDocumentManager;
use teloc::Dependency;

#[derive(Dependency)]
pub struct MetadataService {
    pub metadata_document_manager: MetadataDocumentManager,
}

impl MetadataService {
    pub fn set_current_task_reference(&self, task_id: Option<String>) -> eyre::Result<()> {
        self.metadata_document_manager
            .set_current_task_reference(task_id)
    }

    pub fn get_current_task_reference(&self) -> eyre::Result<Option<String>> {
        self.metadata_document_manager.get_current_task_reference()
    }
}
