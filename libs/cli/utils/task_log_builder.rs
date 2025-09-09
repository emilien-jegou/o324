use chrono::Local;
use colored::*;
use o324_dbus::dto;
use std::fmt::Display;

use crate::utils::{
    display::{LogBuilder, LogType},
    displayable_id::DisplayableId,
    time::{self, ms_to_datetime},
};

pub struct TaskLogBuilder<'a> {
    log_type: LogType,
    message: String,
    task: &'a dto::TaskDto,

    // Flags
    show_id: bool,
    show_name: bool,
    show_project: bool,
    show_tags: bool,
    show_time_period: bool,
    show_start_time: bool,
    show_elapsed: Option<chrono::Duration>,
}

fn format_duration_human(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();

    if secs < 60 {
        return format!("{secs}s");
    }

    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    // Always show seconds for a running task for a "live" feel
    if seconds >= 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }

    parts.join(" ")
}

impl<'a> TaskLogBuilder<'a> {
    pub fn new_for_task(log_type: LogType, verb: &str, task: &'a dto::TaskDto) -> Self {
        Self::new(log_type, format!("{verb} task"), task)
    }

    /// Creates a new builder with a custom title message.
    pub fn new(log_type: LogType, message: impl Display, task: &'a dto::TaskDto) -> Self {
        Self {
            log_type,
            message: message.to_string(),
            task,
            show_id: true,
            show_name: true,
            show_project: false,
            show_tags: false,
            show_time_period: false,
            show_start_time: false,
            show_elapsed: None,
        }
    }

    pub fn with_id(mut self, value: bool) -> Self {
        self.show_id = value;
        self
    }

    pub fn with_name(mut self, value: bool) -> Self {
        self.show_name = value;
        self
    }

    pub fn with_project(mut self) -> Self {
        self.show_project = true;
        self
    }

    pub fn with_tags(mut self) -> Self {
        self.show_tags = true;
        self
    }

    pub fn with_time_period(mut self) -> Self {
        self.show_time_period = true;
        self
    }

    pub fn with_start_time(mut self) -> Self {
        self.show_start_time = true;
        self
    }

    pub fn with_elapsed(mut self, elapsed: chrono::Duration) -> Self {
        self.show_elapsed = Some(elapsed);
        self
    }

    /// Configures the builder to show details for a running task.
    pub fn with_running_details(self, elapsed: chrono::Duration) -> Self {
        self.with_id(true)
            .with_name(true)
            .with_project()
            .with_tags()
            .with_start_time()
            .with_elapsed(elapsed)
    }

    /// Configures the builder to show details for an edited or completed task.
    pub fn with_completed_details(self) -> Self {
        self.with_id(true)
            .with_project()
            .with_tags()
            .with_time_period()
    }

    /// Assembles the LogBuilder in the correct order and prints it.
    pub fn print(self) {
        let mut builder = LogBuilder::new(self.log_type, self.message);

        if self.show_id {
            let display_id = DisplayableId::from(self.task);
            builder = builder.with_branch("ID", display_id);
        }

        if self.show_name {
            builder = builder.with_branch("Name", self.task.task_name.cyan());
        }

        if self.show_project {
            let project_display: Box<dyn Display> = if let Some(p) = &self.task.project {
                Box::new(p.yellow())
            } else {
                Box::new("<none>".italic())
            };
            builder = builder.with_branch("Project", project_display);
        }

        if self.show_tags {
            let tags_display = if !self.task.tags.is_empty() {
                Some(self.task.tags.join(", ").yellow())
            } else {
                None
            };
            builder = builder.with_optional_branch("Tags", tags_display);
        }

        if self.show_time_period {
            let time_display = time::format_time_period_for_display(self.task.start, self.task.end);
            builder = builder.with_branch("Time", time_display.dimmed());
        } else if self.show_start_time {
            if let Ok(start_time_local) =
                ms_to_datetime(self.task.start).map(|dt| dt.with_timezone(&Local))
            {
                let started_str = format!(
                    "{} (on {})",
                    start_time_local.format("%H:%M:%S"),
                    self.task.computer_name
                );
                builder = builder.with_branch("Started", started_str.dimmed());
            }
        }

        // 6. Elapsed Time
        if let Some(elapsed) = self.show_elapsed {
            let elapsed_str = format_duration_human(elapsed);
            builder = builder.with_branch("Elapsed", elapsed_str.bold());
        }

        builder.print();
    }
}
