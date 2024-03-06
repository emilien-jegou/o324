use std::{collections::BTreeSet, path::PathBuf};

use lazy_regex::regex;
use o324_storage_core::Task;
use shaku::{Component, Interface};

use crate::{
    models::{metadata_document::MetadataDocument, task_document::TaskDocument},
    utils::files::{self, find_matching_files},
};

pub trait IMetadataManager: Interface {
    fn get_current(&self) -> eyre::Result<MetadataDocument>;
    fn set_current(&self, meta: MetadataDocument) -> eyre::Result<()>;
    fn save_task_ref(&self, task_id: &str) -> eyre::Result<()>;
    fn delete_task_ref(&self, task_id: &str) -> eyre::Result<()>;
    fn set_current_task(&self, task_id: Option<String>) -> eyre::Result<()>;
    fn recompute(&self) -> eyre::Result<MetadataDocument>;
}

#[derive(Component)]
#[shaku(interface = IMetadataManager)]
pub struct MetadataManager {
    git_storage_path: PathBuf,
    metadata_path: PathBuf,
}

impl IMetadataManager for MetadataManager {
    fn get_current(&self) -> eyre::Result<MetadataDocument> {
        files::read_json_document_as_struct_with_default(&self.metadata_path)
    }

    fn set_current(&self, meta: MetadataDocument) -> eyre::Result<()> {
        files::save_json_document(&self.metadata_path, &meta)?;
        Ok(())
    }

    fn save_task_ref(&self, task_id: &str) -> eyre::Result<()> {
        let mut metadata = self.get_current()?;
        metadata.task_refs.insert(task_id.to_string());
        self.set_current(metadata)?;
        Ok(())
    }

    fn delete_task_ref(&self, task_id: &str) -> eyre::Result<()> {
        let mut metadata = self.get_current()?;
        metadata.task_refs.remove(task_id);
        self.set_current(metadata)?;
        Ok(())
    }

    fn set_current_task(&self, task_id: Option<String>) -> eyre::Result<()> {
        let mut metadata = self.get_current()?;
        metadata.current = task_id;
        self.set_current(metadata)?;
        Ok(())
    }

    fn recompute(&self) -> eyre::Result<MetadataDocument> {
        let re = regex!(r"20\d{2}-\d{2}-\d{2}\.json$");

        // Find and parse all task documents
        let all_task_documents = find_matching_files(&self.git_storage_path, &re)?
            .into_iter()
            .map(|path| {
                let content = std::fs::read_to_string(self.git_storage_path.join(path))?;
                let doc: TaskDocument = serde_json::from_str(&content)?;
                Ok(doc)
            })
            .collect::<eyre::Result<Vec<TaskDocument>>>()?;

        // Extract and combine every tasks
        let all_tasks = all_task_documents
            .into_iter()
            .map(|v| {
                v.tasks
                    .into_iter()
                    .map(|(_, task)| task)
                    .collect::<Vec<Task>>()
            })
            .flatten()
            .collect::<Vec<Task>>();

        let current: Option<String> = all_tasks
            .iter()
            .find(|a| a.end == None)
            .map(|t| t.ulid.clone());

        let task_refs: BTreeSet<String> = all_tasks.iter().map(|t| t.ulid.clone()).collect();

        Ok(MetadataDocument { current, task_refs })
    }
}
