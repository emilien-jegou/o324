use o324_storage_core::{Task, TaskId, TaskUpdate};

#[derive(PartialEq, Debug)]
pub enum TaskChangeObject {
    Created(Task),
    Deleted(TaskId),
    Updated(TaskUpdate),
}

impl TaskChangeObject {
    pub fn try_get_ulid(&self) -> eyre::Result<TaskId> {
        match self {
            Self::Created(task) => Ok(task.ulid.clone()),
            Self::Updated(task) => task.get_ulid(),
            Self::Deleted(ulid) => Ok(ulid.clone()),
        }
    }
}

pub fn build_task_change_object(
    left: &[Task],
    right: &[Task],
) -> eyre::Result<Vec<TaskChangeObject>> {
    let mut results: Vec<TaskChangeObject> = Vec::new();

    // Verify new and updated tasks
    for task in right.iter() {
        if let Some(ref prev) = left.iter().find(|&x| x.ulid == task.ulid) {
            if prev != &task {
                results.push(TaskChangeObject::Updated(TaskUpdate::from_task_diff(
                    prev, task,
                )?));
            }
        } else {
            results.push(TaskChangeObject::Created(task.clone()));
        }
    }

    // Verify deleted tasks
    for task in left.iter() {
        if !right.iter().any(|x| x.ulid == task.ulid) {
            results.push(TaskChangeObject::Deleted(task.ulid.clone()));
        }
    }

    Ok(results)
}

// TODO: test this
pub fn apply_task_change_object(
    left: &mut Vec<Task>,
    right: Vec<TaskChangeObject>,
) -> eyre::Result<()> {
    for change in right.into_iter() {
        match change {
            TaskChangeObject::Deleted(id) => {
                if let Some(index) = left.iter().position(|t| t.ulid == id) {
                    left.remove(index);
                }
            }
            TaskChangeObject::Updated(task) => {
                let id = task.get_ulid()?;
                if let Some(index) = left.iter().position(|t| t.ulid == id) {
                    // Update the element at the found index
                    left[index] = task.merge_with_task(&left[index]);
                }
            }
            TaskChangeObject::Created(task) => {
                left.push(task);
            }
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::tasks_vec;

    use super::*;

    #[test]
    fn no_change_detected() {
        let changes = build_task_change_object(&[], &[]).unwrap();

        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn task_change_create() {
        let le = tasks_vec!([]);
        let ri = tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);

        let res = build_task_change_object(&le, &ri).unwrap();

        let expected = vec![TaskChangeObject::Created(Task {
            ulid: "b2".to_owned(),
            task_name: "0".to_owned(),
            project: None,
            tags: Vec::new(),
            start: 5,
            end: Some(6),
            __version: 0
        })];

        assert_eq!(res, expected);
    }

    #[test]
    fn task_change_update() {
        let le = tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([{"ulid":"b2","task_name":"1","tags":[],"start":5,"end":6}]);

        let res = build_task_change_object(&le, &ri).unwrap();

        let expected = vec![TaskChangeObject::Updated(
            TaskUpdate::default()
                .set_ulid("b2".to_owned())
                .set_task_name("1".to_owned()),
        )];

        assert_eq!(res, expected);
    }

    #[test]
    fn task_change_delete() {
        let le = tasks_vec!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = tasks_vec!([]);

        let res = build_task_change_object(&le, &ri).unwrap();

        let expected = vec![TaskChangeObject::Deleted("b2".to_owned())];

        assert_eq!(res, expected);
    }
}
