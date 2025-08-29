use std::fmt;
use std::str::FromStr;

use colored::Colorize;
use o324_dbus::dto;
use o324_dbus::proxy::O324ServiceProxy;

use crate::utils::command_error;
use crate::utils::displayable_id::DisplayableId;

/// Represents a reference to a task using various methods.
///
/// This can be a specific ID or a special reference using the `@` prefix.
/// Special references include `@current`, `@last`, or a relative historical
/// reference like `@2` (for "two tasks ago").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskRef {
    /// A specific task identifier, e.g., "abc123f".
    Id(String),
    /// A reference to the currently active task. Parsed from "@current".
    Current,
    /// A reference to the most recently completed or referenced task. Parsed from "@last".
    Last,
    /// A reference to a task 'n' steps back in the history. Parsed from "@0", "@1", etc.
    Ago(u32),
}

impl TaskRef {
    pub async fn get_task(
        &self,
        proxy: &O324ServiceProxy<'_>,
    ) -> command_error::Result<dto::TaskDto> {
        match self {
            TaskRef::Id(task_ref) => {
                let by_prefix = proxy.get_task_by_prefix(task_ref.clone()).await?.unpack();

                match by_prefix {
                    dto::TaskByPrefixDto::Single(task_dto) => Ok(task_dto),
                    dto::TaskByPrefixDto::Many(task_dtos) => {
                        let mut error_message = format!(
                            "{} The provided ID '{}' is ambiguous and matches multiple tasks:\n",
                            "✗".red().bold(),
                            task_ref.yellow()
                        );

                        for task in task_dtos {
                            let display_id = DisplayableId::from(&task);
                            let mut parts = vec![
                                format!("ID: {}", display_id.to_string().bold()),
                                format!("Name: '{}'", task.task_name.cyan()),
                            ];

                            if let Some(project) = &task.project {
                                parts.push(format!("Project: {}", project.green()));
                            }

                            if !task.tags.is_empty() {
                                let tags_str = task.tags.join(", ");
                                parts.push(format!("Tags: [{}]", tags_str.blue()));
                            }

                            let task_line = format!("  - {}", parts.join(" | "));
                            error_message.push_str(&task_line);
                            error_message.push('\n');
                        }

                        error_message.push_str("\nPlease use a more specific ID to select a task.");

                        // Propagate the constructed error message, halting the function.
                        Err(eyre::eyre!(error_message))?
                    }
                    dto::TaskByPrefixDto::NotFound => {
                        let hint = match task_ref.as_ref() {
                            "last" => " Perhaps you meant @last?",
                            "current" => " Perhaps you meant @current?",
                            _ => "",
                        };
                        // This also works seamlessly due to the Display implementation.
                        Err(eyre::eyre!(
                            "Task with id '{task_ref}' was not found.{hint}"
                        ))?
                    }
                }
            }
            TaskRef::Current => {
                let r = proxy.list_last_tasks(0, 1).await?;

                let current = r.first().ok_or(eyre::eyre!("No task in history"))?;
                if current.end.is_some() {
                    Err(eyre::eyre!("No task currently running"))?;
                }

                Ok(current.clone())
            }
            TaskRef::Last => Ok(proxy
                .list_last_tasks(0, 1)
                .await?
                .pop()
                .ok_or_else(|| eyre::eyre!("No task to resume"))?),

            TaskRef::Ago(n) => {
                let index = (*n).checked_sub(1).ok_or_else(|| {
                    eyre::eyre!("Provided invalid reference @0, index start at 1.")
                })?;

                let task = proxy
                    .list_last_tasks(index as u64, 1)
                    .await?
                    .pop()
                    .ok_or_else(|| eyre::eyre!("Task not in range: no task at 'ago {}'", n))?;

                Ok(task)
            }
        }
    }
}

/// Allows printing the TaskRef in a user-friendly format.
impl fmt::Display for TaskRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskRef::Id(s) => write!(f, "{}", s),
            TaskRef::Current => write!(f, "@current"),
            TaskRef::Last => write!(f, "@last"),
            TaskRef::Ago(n) => write!(f, "@{}", n),
        }
    }
}

/// Allows parsing a string into a TaskRef.
impl FromStr for TaskRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(suffix) = s.strip_prefix('@') {
            return match suffix.to_lowercase().as_str() {
                "current" => Ok(TaskRef::Current),
                "last" => Ok(TaskRef::Last),
                // It's not a keyword, so try to parse it as a history number.
                num_str => match num_str.parse::<u32>() {
                    Ok(n) => Ok(TaskRef::Ago(n)),
                    Err(_) => Err(format!("Invalid special reference: expected '@current', '@last', or a number like '@2', but found '@{}'.", suffix)),
                },
            };
        }

        if (2..=7).contains(&s.len()) {
            Ok(TaskRef::Id(s.to_string()))
        } else {
            Err(format!(
                "Invalid task reference: '{}'. Expected an ID (2-7 chars) or a special reference like '@current', '@last', or '@2'.",
                s
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id() {
        assert_eq!(
            "abc".parse::<TaskRef>().unwrap(),
            TaskRef::Id("abc".to_string())
        );
        assert_eq!(
            "1234567".parse::<TaskRef>().unwrap(),
            TaskRef::Id("1234567".to_string())
        );
    }

    #[test]
    fn test_parse_id_that_looks_like_old_keyword() {
        // This now works, as the conflict is resolved.
        assert_eq!(
            "current".parse::<TaskRef>().unwrap(),
            TaskRef::Id("current".to_string())
        );
        assert_eq!(
            "last".parse::<TaskRef>().unwrap(),
            TaskRef::Id("last".to_string())
        );
    }

    #[test]
    fn test_parse_special_refs() {
        assert_eq!("@current".parse::<TaskRef>().unwrap(), TaskRef::Current);
        assert_eq!("@CURRENT".parse::<TaskRef>().unwrap(), TaskRef::Current); // case-insensitive
        assert_eq!("@last".parse::<TaskRef>().unwrap(), TaskRef::Last);
        assert_eq!("@LAST".parse::<TaskRef>().unwrap(), TaskRef::Last); // case-insensitive
    }

    #[test]
    fn test_parse_ago() {
        assert_eq!("@0".parse::<TaskRef>().unwrap(), TaskRef::Ago(0));
        assert_eq!("@5".parse::<TaskRef>().unwrap(), TaskRef::Ago(5));
    }

    #[test]
    fn test_parse_invalid() {
        // Invalid length for ID
        assert!("a".parse::<TaskRef>().is_err());
        assert!("abcdefgh".parse::<TaskRef>().is_err());
        // Invalid special reference
        assert!("@".parse::<TaskRef>().is_err());
        assert!("@foo".parse::<TaskRef>().is_err());
        assert!("@ 1".parse::<TaskRef>().is_err());
        assert!("@-1".parse::<TaskRef>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(TaskRef::Id("abc".to_string()).to_string(), "abc");
        assert_eq!(TaskRef::Current.to_string(), "@current");
        assert_eq!(TaskRef::Last.to_string(), "@last");
        assert_eq!(TaskRef::Ago(3).to_string(), "@3");
    }
}
