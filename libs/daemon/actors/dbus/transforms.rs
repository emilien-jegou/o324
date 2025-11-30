use crate::{
    entities::{
        activity::Activity,
        task::{Task, TaskUpdate, TaskUpdateEnd},
    },
    repositories::task::defs::{StartTaskAt, StartTaskInput, TaskAction},
    services::{
        storage_bridge::{DbOperation, DbResult},
        task::TaskWithMeta,
    },
};
use o324_dbus::dto::{self, TaskUpdateEndDto};

// Convert from Core Task -> DTO Task (for sending data out)
impl From<TaskWithMeta> for dto::TaskDto {
    fn from(v: TaskWithMeta) -> Self {
        Self {
            id_prefix: v.prefix,
            __hash: v.task.get_hash(),
            id: v.task.id,
            task_name: v.task.task_name,
            project: v.task.project,
            tags: v.task.tags,
            computer_name: v.task.computer_name,
            start: v.task.start,
            end: v.task.end,
        }
    }
}

use eyre::{eyre, Result}; // Assuming eyre is used for errors

impl TryFrom<dto::StartTaskInputDto> for StartTaskInput {
    type Error = eyre::Report;

    fn try_from(dto: dto::StartTaskInputDto) -> Result<Self, Self::Error> {
        let at = match dto.start {
            Some(start_ts) => {
                let end_ts = match dto.end {
                    Some(dto::TaskUpdateEndDto::Absolute(ts)) => Some(ts),
                    Some(dto::TaskUpdateEndDto::Relative(ms)) => Some(start_ts + ms),
                    None => None,
                };

                Some(StartTaskAt {
                    start: start_ts,
                    end: end_ts,
                })
            }
            None => {
                if dto.end.is_some() {
                    return Err(eyre!(
                        "End time or duration cannot be specified without a start time."
                    ));
                }
                None
            }
        };

        Ok(Self {
            task_name: dto.task_name,
            project: dto.project,
            tags: dto.tags,
            at,
        })
    }
}

impl From<dto::TaskUpdateDto> for TaskUpdate {
    fn from(dto: dto::TaskUpdateDto) -> Self {
        TaskUpdate {
            task_name: dto.task_name.into(),
            project: dto.project.into(),
            tags: dto.tags.into(),
            start: dto.start.into(),
            end: Option::<Option<TaskUpdateEndDto>>::from(dto.end)
                .map(|x| x.map(TaskUpdateEnd::from)),
        }
    }
}

impl From<TaskAction> for dto::TaskActionDto {
    fn from(action: TaskAction) -> Self {
        match action {
            TaskAction::Upsert(task) => dto::TaskActionDto::Upsert(task.into()),
            TaskAction::Delete(task_id) => dto::TaskActionDto::Delete(task_id),
        }
    }
}

impl From<Task> for dto::TaskActionUpsertDto {
    fn from(value: Task) -> Self {
        Self {
            __hash: value.get_hash(),
            id: value.id,
            task_name: value.task_name,
            project: value.project,
            tags: value.tags,
            computer_name: value.computer_name,
            start: value.start,
            end: value.end,
        }
    }
}

// This converts the incoming request DTO into our internal operation enum.
impl TryFrom<dto::DbOperationDto> for DbOperation {
    type Error = String;

    fn try_from(dto: dto::DbOperationDto) -> Result<Self, Self::Error> {
        match dto.operation_type {
            dto::DbOperationTypeDto::ListTables => Ok(DbOperation::ListTables),
            dto::DbOperationTypeDto::ScanTable => {
                // Validate that table_name is present for this operation type.
                if let Some(table_name) = dto.table_name {
                    Ok(DbOperation::ScanTable { table_name })
                } else {
                    Err("ScanTable operation requires a 'table_name'".to_string())
                }
            }
        }
    }
}

impl From<DbResult> for dto::DbResultDto {
    fn from(result: DbResult) -> Self {
        match result {
            DbResult::TableList(tables) => dto::DbResultDto::TableList(tables),
            DbResult::TableRows(rows) => dto::DbResultDto::TableRows(rows),
        }
    }
}

impl From<Activity> for dto::ActivityDto {
    fn from(value: Activity) -> Self {
        Self {
            id: value.id,
            app_name: value.app_name,
            start: value.start,
            last_active: value.last_active,
            computer_name: value.computer_name,
        }
    }
}
