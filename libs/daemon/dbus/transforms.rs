use o324_dbus::dto::TaskActionType;
use zvariant::Optional;

use crate::{
    core::{StartTaskInput, TaskAction},
    dbus::dto,
    storage::task::{Task, TaskUpdate},
};

// Convert from Core Task -> DTO Task (for sending data out)
impl From<Task> for dto::TaskDto {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.clone(),
            __hash: task.get_hash().clone(),
            task_name: task.task_name,
            project: task.project,
            tags: task.tags,
            computer_name: task.computer_name,
            start: task.start,
            end: task.end,
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
