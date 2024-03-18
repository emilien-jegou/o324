use crate::{
    managers::task_document_manager::TaskDocumentManager,
    models::{metadata_document::MetadataDocument, task_document::TaskDocument},
    module::GitService,
    task_actions::{
        repair_unique_current_task::repair_unique_current_task,
        resolve_task_conflict::resolve_tasks_conflict,
    },
};
use git_document_db::SyncConflict;
use o324_storage_core::Task;
use std::collections::BTreeSet;
use teloc::Dependency;

#[derive(Dependency)]
pub struct StorageSyncService {
    storage: GitService,
    task_document_manager: TaskDocumentManager,
}

impl StorageSyncService {
    pub fn compute_document_from_conflict(
        &self,
        conflict: &SyncConflict,
    ) -> eyre::Result<TaskDocument> {
        let left: TaskDocument = conflict.left.to_document()?;
        let right: TaskDocument = conflict.left.to_document()?;
        let previous: Option<TaskDocument> = match &conflict.previous {
            Some(content) => Some(content.to_document()?),
            None => None,
        };

        // Convert task document to vector
        let prev_tasks = previous
            .map(|p| p.tasks.values().cloned().collect::<Vec<Task>>())
            .unwrap_or_default();

        let left_tasks: Vec<Task> = left.tasks.values().cloned().collect();
        let right_tasks: Vec<Task> = right.tasks.values().cloned().collect();

        let tasks = resolve_tasks_conflict(prev_tasks, (left_tasks, right_tasks))?;

        Ok(TaskDocument {
            id: conflict.id.clone(),
            tasks,
        })
    }

    fn heal_non_unique_current_tasks(&self, all_tasks: &[Task]) -> eyre::Result<()> {
        let update_operations = repair_unique_current_task(all_tasks)?;
        for (id, task_update) in update_operations.into_iter() {
            self.task_document_manager.update_task(&id, task_update)?;
        }
        Ok(())
    }

    /// Recompute the metadata file using a list of task data
    fn recompute_new_metadata_document(
        &self,
        all_tasks: &[Task],
    ) -> eyre::Result<MetadataDocument> {
        let current = all_tasks
            .iter()
            .find(|a| a.end.is_none())
            .map(|t| t.ulid.clone());

        let task_refs: BTreeSet<String> = all_tasks.iter().map(|t| t.ulid.clone()).collect();

        Ok(MetadataDocument {
            id: "__metadata".to_string(),
            current,
            task_refs,
        })
    }

    pub fn sync(&self) -> eyre::Result<()> {
        let task_document_regex = self.task_document_manager.get_task_document_regex()?;
        self.storage.0.sync(|conflicts| {
            for conflict in conflicts.iter() {
                if task_document_regex.is_match(&conflict.id) {
                    let task_document = self.compute_document_from_conflict(conflict)?;
                    conflict.save(task_document)?;
                }

                let all_tasks = self.task_document_manager.get_all_tasks()?;

                self.heal_non_unique_current_tasks(&all_tasks)?;
                let meta = self.recompute_new_metadata_document(&all_tasks)?;
                conflict.save(meta)?;
            }

            Ok(())
        })?;
        Ok(())
    }
}
