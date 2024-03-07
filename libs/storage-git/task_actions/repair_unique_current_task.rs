use o324_storage_core::{Task, TaskId, TaskUpdate};

/// Return a list of updates that needs to be performed to ensure that only one current task is
/// active at the time
pub fn repair_unique_current_task(all_tasks: &[Task]) -> eyre::Result<Vec<(TaskId, TaskUpdate)>> {
    let mut current_tasks: Vec<&Task> = all_tasks.iter().filter(|a| a.end.is_none()).collect();
    let mut tasks_update_list: Vec<(TaskId, TaskUpdate)> = Vec::new();

    for task in current_tasks.iter_mut() {
        let nearest_task_start_date = all_tasks
            .iter()
            .filter(|t| {
                if t.start == task.start {
                    return t.ulid > task.ulid;
                }

                t.start > task.start
            })
            .map(|t| t.start)
            .min();

        if let Some(nearest) = nearest_task_start_date {
            tasks_update_list.push((
                task.ulid.clone(),
                TaskUpdate::default().set_end(Some(nearest)),
            ));
        }
    }

    Ok(tasks_update_list)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::tasks_vec;

    fn apply_update_tasks(tasks: &[Task], updates: Vec<(TaskId, TaskUpdate)>) -> Vec<Task> {
        let mut h_tasks: HashMap<String, Task> =
            tasks.iter().cloned().map(|t| (t.ulid.clone(), t)).collect();

        for (task_id, task_update) in updates.into_iter() {
            let row = h_tasks.get(&task_id).unwrap();
            let new_task = task_update.merge_with_task(row);
            h_tasks.insert(task_id, new_task);
        }

        let mut v = h_tasks.values().cloned().collect::<Vec<Task>>();
        v.sort_by_key(|t| t.ulid.clone());
        v
    }

    #[test]
    fn test_repair_unique_current_task_simple() {
        let tasks = tasks_vec!([
           {"ulid": "1", "task_name": "Hello", "tags": [],"start": 5 },
           {"ulid": "2", "task_name": "Hello", "tags": [],"start": 10 },
           {"ulid": "3", "task_name": "Hello", "tags": [],"start": 30,"end": 40 }
        ]);

        let result = repair_unique_current_task(&tasks).unwrap();

        let new_tasks = apply_update_tasks(&tasks, result);

        let expected = tasks_vec!([
           {"ulid": "1", "task_name": "Hello", "tags": [],"start": 5, "end": 10 },
           {"ulid": "2", "task_name": "Hello", "tags": [],"start": 10, "end": 30 },
           {"ulid": "3", "task_name": "Hello", "tags": [],"start": 30,"end": 40 }
        ]);

        assert_eq!(new_tasks, expected);
    }

    #[test]
    fn test_repair_unique_current_task_keep_active() {
        let tasks = tasks_vec!([
           {"ulid": "1", "task_name": "Hello", "tags": [],"start": 5 },
           {"ulid": "2", "task_name": "Hello", "tags": [],"start": 7 },
        ]);

        let result = repair_unique_current_task(&tasks).unwrap();

        let new_tasks = apply_update_tasks(&tasks, result);

        let expected = tasks_vec!([
           {"ulid": "1", "task_name": "Hello", "tags": [],"start": 5, "end": 7 },
           {"ulid": "2", "task_name": "Hello", "tags": [],"start": 7 },
        ]);

        assert_eq!(new_tasks, expected);
    }

    #[test]
    fn test_repair_unique_current_task_same_start() {
        let tasks = tasks_vec!([
           {"ulid": "1", "task_name": "Hello", "tags": [],"start": 5 },
           {"ulid": "2", "task_name": "Hello", "tags": [],"start": 5 },
        ]);

        let result = repair_unique_current_task(&tasks).unwrap();

        let new_tasks = apply_update_tasks(&tasks, result);

        let expected = tasks_vec!([
           {"ulid": "1", "task_name": "Hello", "tags": [],"start": 5, "end": 5 },
           {"ulid": "2", "task_name": "Hello", "tags": [],"start": 5 },
        ]);

        assert_eq!(new_tasks, expected);
    }
}
