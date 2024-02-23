use crate::{core::task::TaskId, Task, TaskUpdate};

#[derive(PartialEq, Debug)]
pub enum TaskActionObject {
    Created(Task),
    Deleted(TaskId),
    Updated(TaskUpdate),
}

pub fn build_task_change_object(
    left: Vec<Task>,
    right: Vec<Task>,
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
        if right.iter().find(|&x| x.ulid == task.ulid).is_none() {
            results.push(TaskActionObject::Deleted(task.ulid.clone()));
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn get_tasks(s: &str) -> Vec<Task> {
        serde_json::from_str::<HashMap<String, Task>>(s)
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    #[test]
    fn no_change_detected() {
        let changes = build_task_change_object(vec![], vec![]).unwrap();

        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn task_action_create() {
        let le = get_tasks(r#"{}"#);
        let ri = get_tasks(r#"{"b2":{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}}"#);

        let res = build_task_change_object(le, ri).unwrap();

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
        let le = get_tasks(r#"{"b2":{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}}"#);
        let ri = get_tasks(r#"{"b2":{"ulid":"b2","task_name":"1","tags":[],"start":5,"end":6}}"#);

        let res = build_task_change_object(le, ri).unwrap();

        let expected = vec![TaskActionObject::Updated(
            TaskUpdate::default()
                .set_ulid("b2".to_owned())
                .set_task_name("1".to_owned()),
        )];

        assert_eq!(res, expected);
    }

    #[test]
    fn task_action_delete() {
        let le = get_tasks(r#"{"b2":{"ulid":"b2","task_name":"0","tags":[],"start":5,"end":6}}"#);
        let ri = get_tasks(r#"{}"#);

        let res = build_task_change_object(le, ri).unwrap();

        let expected = vec![TaskActionObject::Deleted("b2".to_owned())];

        assert_eq!(res, expected);
    }
}
