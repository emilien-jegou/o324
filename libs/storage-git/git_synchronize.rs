use lazy_regex::regex;
use o324_storage_core::{Task, TaskId};

use crate::{
    git_actions::{self, rebase::Rebase},
    managers::metadata_manager::IMetadataManager,
    models::task_document::TaskDocument,
};

use super::task_action::{apply_task_action_object, build_task_action_object, TaskActionObject};
use std::collections::{BTreeMap, HashSet};

pub fn rebase_action<'repo>(
    metadata_manager: &Box<dyn IMetadataManager>,
    rebase: &mut Rebase<'repo>,
) -> eyre::Result<()> {
    let task_file_regex = regex!("20[0-9]{2}-[0-9]{2}-[0-9]{2}\\.json");

    //// Apply each rebase operation
    for op in rebase.iter() {
        let mut operation = op?;
        let mut conflict = operation.get_conflict()?;

        for file in conflict.files.iter_mut() {
            if task_file_regex.is_match(&file.relative_file_path) {
                let previous: Option<TaskDocument> = match &file.previous {
                    Some(content) => Some(serde_json::from_str(content)?),
                    None => None,
                };
                let left: TaskDocument = serde_json::from_str(&file.left)?;
                let right: TaskDocument = serde_json::from_str(&file.right)?;

                let mut prev_tasks = previous
                    .map(|p| p.tasks.values().cloned().collect::<Vec<Task>>())
                    .unwrap_or_default();
                let left_tasks: Vec<Task> = left.tasks.values().cloned().collect();
                let right_tasks: Vec<Task> = right.tasks.values().cloned().collect();

                let left_tao = build_task_action_object(&prev_tasks, &left_tasks)?;
                let right_tao = build_task_action_object(&prev_tasks, &right_tasks)?;

                // Collect Task IDs from the `right` vector
                let right_ids: HashSet<TaskId> = right_tao
                    .iter()
                    .map(|action| {
                        let result: eyre::Result<TaskId> = match action {
                            TaskActionObject::Created(task) => Ok(task.ulid.clone()),
                            TaskActionObject::Updated(task) => task.get_ulid(),
                            TaskActionObject::Deleted(ulid) => Ok(ulid.clone()),
                        };
                        result
                    })
                    .collect::<eyre::Result<HashSet<TaskId>>>()?;

                // Don't remove deleted task that got updated recently
                let filtered_left_tao: Vec<TaskActionObject> = left_tao
                    .into_iter()
                    .filter(|action| match action {
                        TaskActionObject::Created(task) => !right_ids.contains(&task.ulid),
                        TaskActionObject::Deleted(task_ulid) => !right_ids.contains(task_ulid),
                        _ => true,
                    })
                    .collect();

                // Apply the last commit changes first, following
                // CRDT's reconciliation principles
                if conflict.left_commit.timestamp > conflict.right_commit.timestamp {
                    apply_task_action_object(&mut prev_tasks, right_tao)?;
                    apply_task_action_object(&mut prev_tasks, filtered_left_tao)?;
                } else {
                    apply_task_action_object(&mut prev_tasks, filtered_left_tao)?;
                    apply_task_action_object(&mut prev_tasks, right_tao)?;
                }

                let mut tasks: BTreeMap<String, Task> = prev_tasks
                    .into_iter()
                    .map(|t| (t.ulid.clone(), t))
                    .collect();

                let tasks_start_dates = tasks.values().map(|t| t.start).collect::<Vec<u64>>();

                // If they are multiple current tasks we will need
                // to choose the latest one.
                let mut current_tasks = tasks
                    .values_mut()
                    .filter(|task| task.end.is_none())
                    .collect::<Vec<&mut Task>>();

                if let Some((index, _)) = current_tasks
                    .iter()
                    .enumerate()
                    .max_by_key(|&(_, task)| task.start)
                {
                    current_tasks.remove(index);
                }

                for task in current_tasks.iter_mut() {
                    let nearest_task_start_date = tasks_start_dates
                        .iter()
                        .filter(|start| **start > task.start)
                        .min()
                        .ok_or_else(|| eyre::eyre!("An unexpected situation occured"))?;

                    task.end = Some(*nearest_task_start_date);
                }

                file.resolve(&serde_json::to_string_pretty(&TaskDocument { tasks })?);
            } else if file.relative_file_path != "__metadata.json" {
                Err(eyre::eyre!(
                    "Encountered unresolveable conflict on file '{}', manually intervention required",
                    file.relative_file_path
                ))?;
            }
        }

        // Write all tasks changes
        conflict.write_changes()?;

        // Recompute metadata file using new tasks
        if let Some(file) = conflict
            .files
            .iter_mut()
            .find(|a| a.relative_file_path == "__metadata.json")
        {
            let document = metadata_manager.recompute()?;
            file.resolve(&serde_json::to_string_pretty(&document)?);
        }

        // Write remanining metadata changes if any, and stage all files
        conflict.write_changes()?.stage_all()?;

        operation.commit_changes()?;
    }
    Ok(())
}

pub fn rebase_with_auto_resolve(
    metadata_manager: &Box<dyn IMetadataManager>,
    repository: &git2::Repository,
) -> eyre::Result<()> {
    let mut rebase = git_actions::rebase_current_branch(repository)?;

    match rebase_action(metadata_manager, &mut rebase) {
        Ok(()) => Ok(rebase.finalize()?),
        Err(e) => {
            rebase.abort()?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {}
