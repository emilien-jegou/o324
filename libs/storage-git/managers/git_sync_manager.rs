use o324_storage_core::Task;
use shaku::{Component, Interface};
use std::{collections::BTreeSet, sync::Arc};

use crate::{
    git_actions::{
        self,
        rebase::{Conflict, ConflictFile, Rebase},
    },
    managers::{
        config_manager::IConfigManager, metadata_document_manager::IMetadataDocumentManager,
    },
    models::{metadata_document::MetadataDocument, task_document::TaskDocument},
    module::TaskDocumentStorage,
    task_actions::{
        repair_unique_current_task::repair_unique_current_task,
        resolve_task_conflict::resolve_tasks_conflict,
    },
};

use super::task_document_manager::ITaskDocumentManager;

pub trait IGitSyncManager: Interface {
    fn rebase_with_auto_resolve(&self) -> eyre::Result<()>;
}

#[derive(Component)]
#[shaku(interface = IGitSyncManager)]
pub struct GitSyncManager {
    #[shaku(inject)]
    metadata_document_manager: Arc<dyn IMetadataDocumentManager>,
    #[shaku(inject)]
    config: Arc<dyn IConfigManager>,
    #[shaku(inject)]
    task_storage: Arc<TaskDocumentStorage>,
    #[shaku(inject)]
    task_manager: Arc<dyn ITaskDocumentManager>,
    #[shaku(inject)]
    task_document_manager: Arc<dyn ITaskDocumentManager>,
}

impl GitSyncManager {
    pub fn compute_document_from_conflict(
        &self,
        conflict: &Conflict,
        file: &ConflictFile,
    ) -> eyre::Result<TaskDocument> {
        // Parse files contents
        let previous: Option<TaskDocument> = match &file.previous {
            Some(content) => Some(self.task_storage.from_str(content)?),
            None => None,
        };
        let left = self.task_storage.from_str(&file.left)?;
        let right = self.task_storage.from_str(&file.right)?;

        // Convert task document to vector
        let prev_tasks = previous
            .map(|p| p.tasks.values().cloned().collect::<Vec<Task>>())
            .unwrap_or_default();

        let left_tasks: Vec<Task> = left.tasks.values().cloned().collect();
        let right_tasks: Vec<Task> = right.tasks.values().cloned().collect();

        let tasks = if conflict.left_commit.timestamp < conflict.right_commit.timestamp {
            resolve_tasks_conflict(prev_tasks, (left_tasks, right_tasks))?
        } else {
            resolve_tasks_conflict(prev_tasks, (right_tasks, left_tasks))?
        };

        Ok(TaskDocument { tasks })
    }

    fn heal_non_unique_current_tasks(&self, all_tasks: &[Task]) -> eyre::Result<()> {
        let update_operations = repair_unique_current_task(all_tasks)?;
        for (id, task_update) in update_operations.into_iter() {
            self.task_manager.update_task(&id, task_update)?;
        }
        Ok(())
    }

    /// Recompute the metadata file using a list of task data
    fn recompute_new_metadata_document(&self, all_tasks: &[Task]) -> eyre::Result<()> {
        let current = all_tasks
            .iter()
            .find(|a| a.end.is_none())
            .map(|t| t.ulid.clone());

        let task_refs: BTreeSet<String> = all_tasks.iter().map(|t| t.ulid.clone()).collect();
        let document = MetadataDocument { current, task_refs };

        self.metadata_document_manager.set_document(document)?;
        Ok(())
    }

    pub fn rebase_action(&self, rebase: &mut Rebase<'_>) -> eyre::Result<()> {
        let task_file_regex = self.task_document_manager.get_task_document_regex()?;

        for op in rebase.iter() {
            let mut operation = op?;
            let conflict = operation.get_conflict()?;

            let task_files = conflict
                .files
                .iter()
                .filter(|file| task_file_regex.is_match(&file.relative_file_path));

            for file in task_files {
                let task_document = self.compute_document_from_conflict(&conflict, file)?;
                self.task_storage
                    .write(&file.relative_file_path, &task_document)?;
            }

            let all_tasks = self.task_document_manager.get_all_tasks()?;
            self.heal_non_unique_current_tasks(&all_tasks)?;
            self.recompute_new_metadata_document(&all_tasks)?;
            conflict.stage_file("__metadata.json")?;

            conflict.stage_conflicted()?;
            operation.commit_changes()?;
        }

        Ok(())
    }
}

impl IGitSyncManager for GitSyncManager {
    fn rebase_with_auto_resolve(&self) -> eyre::Result<()> {
        let repository = git2::Repository::open(self.config.get_repository_path())?;
        let mut rebase = git_actions::rebase_current_branch(&repository)?;

        match self.rebase_action(&mut rebase) {
            Ok(()) => Ok(rebase.finalize()?),
            Err(e) => {
                rebase.abort()?;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utilities::{create_repository_test_setup, get_test_module};
    use shaku::HasComponent;

    #[test]
    fn empty_inputs() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();
        let module = get_test_module(local1, origin);
        let git_sync_manager: &dyn IGitSyncManager = module.resolve_ref();

        git_sync_manager.rebase_with_auto_resolve().unwrap();
    }
}
