use colored::*;
use o324_dbus::dto::TaskDto;
use std::fmt::Display;

/// Represents an ID that has been processed and is ready for formatted display.
/// The complex slicing and length logic is handled during construction.
#[derive(Debug, Clone)]
pub struct DisplayableId {
    /// The original, complete ID string.
    pub full_id: String,
    /// The pre-calculated part of the ID that should be highlighted.
    unique_part: String,
    /// The pre-calculated part of the ID that should be dimmed.
    dimmed_part: String,
}

impl DisplayableId {
    /// Creates a new `DisplayableId` by calculating the parts to be displayed.
    pub fn new(full_id: String, unique_prefix_len: usize) -> Self {
        let desired_display_len = std::cmp::max(8, unique_prefix_len);
        let final_display_len = std::cmp::min(desired_display_len, full_id.len());
        let final_unique_len = std::cmp::min(unique_prefix_len, final_display_len);
        let unique_part = full_id[..final_unique_len].to_string();
        let dimmed_part = full_id[final_unique_len..final_display_len].to_string();

        Self {
            full_id,
            unique_part,
            dimmed_part,
        }
    }
}

impl From<&TaskDto> for DisplayableId {
    fn from(task: &TaskDto) -> Self {
        // The From trait now simply delegates to our main constructor.
        Self::new(task.id.clone(), task.id_prefix.len())
    }
}

impl Display for DisplayableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The Display implementation is now extremely simple!
        // It only describes *how* to format the pre-calculated parts.
        write!(
            f,
            "{}{}",
            self.unique_part.yellow().bold(),
            self.dimmed_part.dimmed()
        )
    }
}
