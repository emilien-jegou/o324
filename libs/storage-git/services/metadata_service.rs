use crate::managers::metadata_document_manager::MetadataDocumentManager;
use git_document_db::SharedQueryRunner;
use teloc::Dependency;

#[derive(Dependency)]
pub struct MetadataService {}

pub struct MetadataServiceLoaded<'a> {
    metadata_document_manager: MetadataDocumentManager<'a>,
}

impl<'a> MetadataService {
    pub fn load(&'a self, query_runner: &'a SharedQueryRunner<'a>) -> MetadataServiceLoaded<'a> {
        MetadataServiceLoaded {
            metadata_document_manager: MetadataDocumentManager::load(query_runner),
        }
    }
}

impl<'a> MetadataServiceLoaded<'a> {
    pub fn set_current_task_reference(&self, task_id: Option<String>) -> eyre::Result<()> {
        self.metadata_document_manager
            .set_current_task_reference(task_id)
    }

    pub fn get_current_task_reference(&self) -> eyre::Result<Option<String>> {
        self.metadata_document_manager.get_current_task_reference()
    }
}
