use o324_dbus::dto::{self, TaskActionType};
use zvariant::Optional;

use crate::{
    core::storage::{DbOperation, DbResult},
    entities::task::{Task, TaskUpdate},
    services::task::{StartTaskInput, TaskAction},
};

// Convert from Core Task -> DTO Task (for sending data out)
impl Task {
    pub fn into_dto(self, id_prefix: String) -> dto::TaskDto {
        dto::TaskDto {
            id_prefix,
            id: self.id,
            __hash: self.__hash,
            task_name: self.task_name,
            project: self.project,
            tags: self.tags,
            computer_name: self.computer_name,
            start: self.start,
            end: self.end,
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

// FIX: Updated the conversion to build the new TaskActionDto struct.
// Convert from Core TaskAction -> DTO TaskAction (for sending data out)
impl From<TaskAction> for dto::TaskActionDto {
    fn from(action: TaskAction) -> Self {
        match action {
            TaskAction::Upsert(task) => Self {
                action_type: TaskActionType::Upsert,
                upsert_action: Optional::from(Some(task.into())),
                delete_action: Optional::from(None),
            },
            TaskAction::Delete(id) => Self {
                action_type: TaskActionType::Delete,
                upsert_action: Optional::from(None),
                delete_action: Optional::from(Some(id)),
            },
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

// This converts our internal result enum into the DTO for the response.
impl From<DbResult> for dto::DbResultDto {
    fn from(result: DbResult) -> Self {
        match result {
            DbResult::TableList(tables) => dto::DbResultDto {
                result_type: dto::DbResultTypeDto::TableList,
                table_list: Some(tables),
                table_rows: None,
                error: None,
            },
            DbResult::TableRows(rows) => dto::DbResultDto {
                result_type: dto::DbResultTypeDto::TableRows,
                table_list: None,
                table_rows: Some(rows),
                error: None,
            },
        }
    }
}
