use o324_storage_core::{Task, TaskId, TaskUpdate};

#[derive(PartialEq, Debug)]
pub enum TaskActionObject {
    Created(Task),
    Deleted(TaskId),
    Updated(TaskUpdate),
}

pub fn build_task_action_object(
    left: &Vec<Task>,
    right: &Vec<Task>,
) -> eyre::Result<Vec<TaskActionObject>> {
    let mut results: Vec<TaskActionObject> = Vec::new();

    // Verify new and updated tasks
    for task in right.iter() {
        if let Some(ref prev) = left.iter().find(|&x| x.ulid == task.ulid) {
            if prev != &task {
                results.push(TaskActionObject::Updated(TaskUpdate::from_task_diff(
                    prev, task,
                )?));
            }
        } else {
            results.push(TaskActionObject::Created(task.clone()));
        }
    }

    // Verify deleted tasks
    for task in left.iter() {
        if !right.iter().any(|x| x.ulid == task.ulid) {
            results.push(TaskActionObject::Deleted(task.ulid.clone()));
        }
    }

    Ok(results)
}

// TODO: test this
pub fn apply_task_action_object(
    left: &mut Vec<Task>,
    right: Vec<TaskActionObject>,
) -> eyre::Result<()> {
    for action in right.into_iter() {
        match action {
            TaskActionObject::Deleted(id) => {
                if let Some(index) = left.iter().position(|t| t.ulid == id) {
                    left.remove(index);
                }
            }
            TaskActionObject::Updated(task) => {
                let id = task.get_ulid()?;
                if let Some(index) = left.iter().position(|t| t.ulid == id) {
                    // Update the element at the found index
                    left[index] = task.merge_with_task(&left[index]);
                }
            }
            TaskActionObject::Created(task) => {
                left.push(task);
            }
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! get_tasks {
        ($($json:tt)*) => {{
            let val = ::serde_json::json!($($json)*);
            let data: Vec<Task> = ::serde_json::from_value(val).unwrap();
            data
        }};
    }

    #[test]
    fn no_change_detected() {
        let changes = build_task_action_object(&vec![], &vec![]).unwrap();

        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn task_action_create() {
        let le = get_tasks!([]);
        let ri = get_tasks!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);

        let res = build_task_action_object(&le, &ri).unwrap();

        let expected = vec![TaskActionObject::Created(Task {
            ulid: "b2".to_owned(),
            task_name: "0".to_owned(),
            project: None,
            tags: Vec::new(),
            start: 5,
            end: Some(6),
        })];

        assert_eq!(res, expected);
    }

    #[test]
    fn task_action_update() {
        let le = get_tasks!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = get_tasks!([{"ulid":"b2","task_name":"1","tags":[],"start":5,"end":6}]);

        let res = build_task_action_object(&le, &ri).unwrap();

        let expected = vec![TaskActionObject::Updated(
            TaskUpdate::default()
                .set_ulid("b2".to_owned())
                .set_task_name("1".to_owned()),
        )];

        assert_eq!(res, expected);
    }

    #[test]
    fn task_action_delete() {
        let le = get_tasks!([{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}]);
        let ri = get_tasks!([]);

        let res = build_task_action_object(&le, &ri).unwrap();

        let expected = vec![TaskActionObject::Deleted("b2".to_owned())];

        assert_eq!(res, expected);
    }
}
