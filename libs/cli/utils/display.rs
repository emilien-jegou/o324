use colored::*;
use std::fmt::Display;

/// Defines the type of log message to determine the icon and color scheme.
pub enum LogType {
    Success,
    Info,
    /// For the 'start' command.
    Start,
    /// For the 'status' command.
    Status,
    /// For the 'stop' command.
    Stop,
}

/// A builder for creating structured, tree-like log messages.
pub struct LogBuilder<'a> {
    log_type: LogType,
    message: String,
    details: Vec<(&'a str, Box<dyn Display>)>,
}

impl<'a> LogBuilder<'a> {
    /// Creates a new LogBuilder with a primary message.
    pub fn new(log_type: LogType, message: impl Display) -> Self {
        Self {
            log_type,
            message: message.to_string(),
            details: Vec::new(),
        }
    }

    /// Adds a new detail line (a "branch") to the log message.
    /// This method can be chained.
    ///
    /// # Arguments
    /// * `label` - The static string label for the detail (e.g., "ID", "Project").
    /// * `value` - Any type that implements `Display`. It will be boxed.
    pub fn with_branch(mut self, label: &'a str, value: impl Display + 'static) -> Self {
        self.details.push((label, Box::new(value)));
        self
    }

    /// Conditionally adds a branch if the `value` is `Some`.
    /// This is a convenience method for handling `Option` types.
    pub fn with_optional_branch<T: Display + 'static>(
        self,
        label: &'a str,
        value: Option<T>,
    ) -> Self {
        if let Some(val) = value {
            self.with_branch(label, val)
        } else {
            self
        }
    }

    /// Consumes the builder and prints the formatted message to the console.
    pub fn print(self) {
        let (symbol, color) = match self.log_type {
            LogType::Success => ("✔", "green"),
            LogType::Info => ("ℹ", "blue"),
            LogType::Start => ("⚡️", "yellow"),
            LogType::Status => ("❯", "blue"),
            LogType::Stop => ("✔", "green"),
        };

        // Print the main header line with a preceding newline for spacing.
        println!(
            "\n{} {}",
            symbol.color(color).bold(),
            self.message.color(color).bold()
        );

        let count = self.details.len();
        if count == 0 {
            return;
        }

        // Print each detail with the appropriate tree connector.
        for (i, (label, value)) in self.details.iter().enumerate() {
            let prefix = if i == count - 1 { "  ╰─" } else { "  ├─" };
            let padded_label = format!("{label}:");
            // Pad to 9 to align with "Duration:" and "Started:" from other commands
            println!("{} {:<9} {}", prefix.dimmed(), padded_label.bold(), value);
        }
    }
}

