enum DiffMarker {
    Left,
    Right,
    Common,
}

pub fn extract_diff_from_conflict(conflict_text: &str) -> (String, String) {
    let mut left_changes: Vec<&str> = Vec::new();
    let mut right_changes: Vec<&str> = Vec::new();
    let mut marker = DiffMarker::Common; // Start with local changes as the default

    let lines = conflict_text.lines();
    for line in lines {
        if line.starts_with("<<<<<<<") {
            marker = DiffMarker::Left;
        } else if line.starts_with("=======") {
            marker = DiffMarker::Right;
        } else if line.starts_with(">>>>>>>") {
            marker = DiffMarker::Common;
        } else {
            match marker {
                DiffMarker::Left => left_changes.push(line),
                DiffMarker::Right => right_changes.push(line),
                DiffMarker::Common => {
                    left_changes.push(line);
                    right_changes.push(line);
                }
            }
        }
    }

    (left_changes.join("\n"), right_changes.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_conflict() {
        let conflict_text = r#"<<<<<<< HEAD
local change
=======
remote change
>>>>>>>"#;
        let versions = extract_diff_from_conflict(conflict_text);
        assert_eq!(
            versions,
            ("local change".to_string(), "remote change".to_string())
        );
    }

    #[test]
    fn test_multiple_conflicts() {
        let conflict_text = r#"<<<<<<< HEAD
local change 1
=======
remote change 1
>>>>>>>
<<<<<<< HEAD
local change 2
=======
remote change 2
>>>>>>>"#;
        let versions = extract_diff_from_conflict(conflict_text);
        assert_eq!(
            versions,
            (
                "local change 1\nlocal change 2".to_string(),
                "remote change 1\nremote change 2".to_string(),
            )
        );
    }

    #[test]
    fn test_no_conflict() {
        let conflict_text = r#"Some regular content without conflict markers."#;
        let versions = extract_diff_from_conflict(conflict_text);
        assert_eq!(
            versions,
            (
                "Some regular content without conflict markers.".to_string(),
                "Some regular content without conflict markers.".to_string()
            )
        );
    }

    #[test]
    fn test_with_text_before_conflict() {
        let conflict_text = r#"Before
<<<<<<< HEAD
local change
=======
remote change
>>>>>>>"#;
        let versions = extract_diff_from_conflict(conflict_text);
        assert_eq!(
            versions,
            (
                "Before\nlocal change".to_string(),
                "Before\nremote change".to_string()
            )
        );
    }

    #[test]
    fn test_with_text_after_conflict() {
        let conflict_text = r#"<<<<<<< HEAD
local change
=======
remote change
>>>>>>>
After
"#;
        let versions = extract_diff_from_conflict(conflict_text);
        assert_eq!(
            versions,
            (
                "local change\nAfter".to_string(),
                "remote change\nAfter".to_string()
            )
        );
    }

    #[test]
    fn test_with_text_in_between_conflicts() {
        let conflict_text = r#"<<<<<<< HEAD
local change 1
=======
remote change 1
>>>>>>>
In Between
<<<<<<< HEAD
local change 2
=======
remote change 2
>>>>>>>
"#;
        let versions = extract_diff_from_conflict(conflict_text);
        assert_eq!(
            versions,
            (
                "local change 1\nIn Between\nlocal change 2".to_string(),
                "remote change 1\nIn Between\nremote change 2".to_string()
            )
        );
    }
}
