use o324_dbus::dto::{self, TaskActionType};
use zvariant::Optional;

use crate::{
    entities::task::{Task, TaskUpdate},
    repositories::task::defs::{StartTaskInput, TaskAction},
    services::{
        storage_bridge::{DbOperation, DbResult},
        task::TaskWithMeta,
    },
};

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

// Convert from DTO StartTaskInput -> Core StartTaskInput (for receiving data)
impl From<dto::StartTaskInputDto> for StartTaskInput {
    fn from(dto: dto::StartTaskInputDto) -> Self {
        Self {
            task_name: dto.task_name,
            project: dto.project,
            tags: dto.tags,
        }
    }
}

// Convert from DTO TaskUpdate -> Core TaskUpdate (for receiving data)
impl From<dto::TaskUpdateDto> for TaskUpdate {
    fn from(dto: dto::TaskUpdateDto) -> Self {
        TaskUpdate::default()
            .set_opt_task_name(dto.task_name)
            .set_opt_project(dto.project)
            .set_opt_tags(dto.tags)
            .set_opt_start(dto.start)
            .set_opt_end(dto.end)
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
