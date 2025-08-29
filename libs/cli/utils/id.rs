use std::fmt;
use std::str::FromStr;

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
