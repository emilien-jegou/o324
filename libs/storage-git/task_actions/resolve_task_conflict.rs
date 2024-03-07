use super::task_change::{apply_task_change_object, build_task_change_object, TaskChangeObject};
use o324_storage_core::{Task, TaskId};
use std::collections::{BTreeMap, HashSet};

pub fn resolve_tasks_conflict(
    mut previous: Vec<Task>,
    conflict: (Vec<Task>, Vec<Task>),
) -> eyre::Result<BTreeMap<String, Task>> {
    let left_tao = build_task_change_object(&previous, &conflict.0)?;
    let right_tao = build_task_change_object(&previous, &conflict.1)?;

    // Collect Task IDs from the `right` vector
    let right_ids: HashSet<TaskId> = right_tao
        .iter()
        .map(TaskChangeObject::try_get_ulid)
        .collect::<eyre::Result<HashSet<TaskId>>>()?;

    // Don't remove deleted task that got updated recently
    let filtered_left_tao: Vec<TaskChangeObject> = left_tao
        .into_iter()
        .filter(|action| match action {
            TaskChangeObject::Created(task) => !right_ids.contains(&task.ulid),
            TaskChangeObject::Deleted(task_ulid) => !right_ids.contains(task_ulid),
            _ => true,
        })
        .collect();

    apply_task_change_object(&mut previous, filtered_left_tao)?;
    apply_task_change_object(&mut previous, right_tao)?;

    Ok(previous.into_iter().map(|t| (t.ulid.clone(), t)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks_vec;
    use sugars::btmap;

    #[test]
    fn empty_inputs() {
        let pr = tasks_vec!([]);
        let le = tasks_vec!([]);
        let ri = tasks_vec!([]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();
        assert_eq!(result, btmap![])
    }

    #[test]
    fn adding_task_0() {
        let pr = tasks_vec!([]);
        let le = tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();
        let data: Vec<Task> = result.values().cloned().collect();

        assert_eq!(
            data,
            tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}])
        )
    }

    #[test]
    fn adding_task_1() {
        let pr = tasks_vec!([]);
        let le = tasks_vec!([]);
        let ri = tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();
        let data: Vec<Task> = result.values().cloned().collect();

        assert_eq!(
            data,
            tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}])
        )
    }

    #[test]
    fn adding_task_2() {
        let pr = tasks_vec!([]);
        let le = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        let data: Vec<Task> = result.values().cloned().collect();

        assert_eq!(
            data,
            tasks_vec!([
                {"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6},
                {"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}
            ])
        )
    }

    #[test]
    fn deleting_task_0() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([]);
        let ri = tasks_vec!([]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        assert_eq!(result, btmap![])
    }

    #[test]
    fn deleting_task_1() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        assert_eq!(result, btmap![])
    }

    #[test]
    fn deleting_task_2() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([]);
        let ri = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        assert_eq!(result, btmap![])
    }

    #[test]
    fn update_task_0() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([{"ulid":"b1","task_name":"NEW","tags":[],"start":5,"end":6}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        let data: Vec<Task> = result.values().cloned().collect();
        assert_eq!(
            data,
            tasks_vec!([{"ulid":"b1","task_name":"NEW","tags":[],"start":5,"end":6}])
        )
    }

    #[test]
    fn update_task_1() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([{"ulid":"b1","task_name":"NEW","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        let data: Vec<Task> = result.values().cloned().collect();
        assert_eq!(
            data,
            tasks_vec!([{"ulid":"b1","task_name":"NEW","tags":[],"start":5,"end":6}])
        )
    }

    #[test]
    fn update_task_2() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([{"ulid":"b1","task_name":"LEFT","tags":["LEFT"],"start":9,"end":9}]);
        let ri =
            tasks_vec!([{"ulid":"b1","task_name":"RIGHT","tags":["RIGHT"],"start":10,"end":10}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        let data: Vec<Task> = result.values().cloned().collect();
        assert_eq!(
            data,
            tasks_vec!([{"ulid":"b1","task_name":"RIGHT","tags":["RIGHT"],"start":10,"end":10}])
        )
    }

    #[test]
    fn update_task_with_deletion_0() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([]);
        let ri = tasks_vec!([{"ulid":"b1","task_name":"UPDATE","tags":[],"start":5,"end":6}]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        let data: Vec<Task> = result.values().cloned().collect();
        assert_eq!(
            data,
            tasks_vec!([{"ulid":"b1","task_name":"UPDATE","tags":[],"start":5,"end":6}])
        )
    }

    #[test]
    fn update_task_with_deletion_1() {
        let pr = tasks_vec!([{"ulid":"b1","task_name":"0","tags":[],"start":5,"end":6}]);
        let le = tasks_vec!([{"ulid":"b1","task_name":"UPDATE","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([]);

        let result = super::resolve_tasks_conflict(pr, (le, ri)).unwrap();

        let data: Vec<Task> = result.values().cloned().collect();
        assert_eq!(data, tasks_vec!([]))
    }
}
