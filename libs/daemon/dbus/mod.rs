use crate::config::Config;
use o324_storage::{Entity, Id};
use serde::{Deserialize, Serialize};
use std::{error::Error, future::pending, sync::Arc};
use zbus::{connection, fdo, interface};
use zvariant::Type;

//=========================================================================================
// D-Bus Data Transfer Objects (DTOs)
// These structs are what zbus serializes/deserializes. They are separate from the
// internal `core` structs, providing a stable API contract.
//=========================================================================================

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct TaskDto {
    pub __id: Id,
    pub __hash: Option<String>,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub computer_name: String,
    pub start: u64,
    pub end: Option<u64>,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct TaskUpdateDto {
    pub task_name: Option<String>,
    pub project: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub start: Option<u64>,
    pub end: Option<Option<u64>>,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct StartTaskInputDto {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub computer_name: String,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub enum TaskActionDto {
    Upsert(TaskDto),
    Delete(Id),
}

//=========================================================================================
// Conversions (Core types <-> DTOs)
//=========================================================================================

// Convert from Core Task -> DTO Task (for sending data out)
impl From<core::storage::task::Task> for TaskDto {
    fn from(task: core::storage::task::Task) -> Self {
        Self {
            __id: task.get_id().clone(),
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
impl From<StartTaskInputDto> for core::StartTaskInput {
    fn from(dto: StartTaskInputDto) -> Self {
        Self {
            task_name: dto.task_name,
            project: dto.project,
            tags: dto.tags,
            computer_name: dto.computer_name,
        }
    }
}

// Convert from DTO TaskUpdate -> Core TaskUpdate (for receiving data)
impl From<TaskUpdateDto> for core::storage::task::TaskUpdate {
    fn from(dto: TaskUpdateDto) -> Self {
        Self {
            task_name: dto.task_name,
            project: dto.project,
            tags: dto.tags,
            start: dto.start,
            end: dto.end,
        }
    }
}

// Convert from Core TaskAction -> DTO TaskAction (for sending data out)
impl From<core::TaskAction> for TaskActionDto {
    fn from(action: core::TaskAction) -> Self {
        match action {
            core::TaskAction::Upsert(task) => TaskActionDto::Upsert(task.into()),
            core::TaskAction::Delete(id) => TaskActionDto::Delete(id),
        }
    }
}

//=========================================================================================
// D-Bus Service Implementation
//=========================================================================================

/// The D-Bus service struct that wraps our application's Core.
pub struct O324Service {
    core: Arc<Core>,
}

#[interface(name = "org.o324.Service1")]
impl O324Service {
    async fn start_new_task(&self, input: StartTaskInputDto) -> fdo::Result<Vec<TaskActionDto>> {
        let core_result = self.core.start_new_task(input.into()).await;
        core_result
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn stop_current_task(&self) -> fdo::Result<Vec<TaskActionDto>> {
        self.core
            .stop_current_task()
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn cancel_current_task(&self) -> fdo::Result<Vec<TaskActionDto>> {
        self.core
            .cancel_current_task()
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn delete_task(&self, task_id: String) -> fdo::Result<Vec<TaskActionDto>> {
        self.core
            .delete_task(task_id)
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn edit_task(
        &self,
        task_ref_str: String,
        update: TaskUpdateDto,
    ) -> fdo::Result<Vec<TaskActionDto>> {
        let task_ref: TaskRef = task_ref_str
            .parse()
            .map_err(|e| fdo::Error::InvalidArgs(e))?;

        self.core
            .edit_task(task_ref, update.into())
            .await
            .map(|actions| actions.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn list_last_tasks(&self, count: u64) -> fdo::Result<Vec<TaskDto>> {
        self.core
            .list_last_tasks(count)
            .await
            .map(|tasks| tasks.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn list_task_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> fdo::Result<Vec<TaskDto>> {
        self.core
            .list_task_range(start_timestamp, end_timestamp)
            .await
            .map(|tasks| tasks.into_iter().map(Into::into).collect())
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

/// The main entry point for the D-Bus service daemon.
pub async fn start_service(core: &Core) -> Result<(), Box<dyn Error>> {
    // 1. Initialize your application's core logic
    //let config = Config::load_from_default_location()?;
    let core = Core::try_new(&config)?;
    let service = O324Service {
        core: Arc::new(core),
    };

    println!("Starting D-Bus service...");

    // 2. Build and run the D-Bus connection
    let _conn = connection::Builder::session()?
        .name("org.o324.Service")?
        .serve_at("/org/o324/Service", service)?
        .build()
        .await?;

    println!("D-Bus service running. Waiting for calls.");

    // 3. Keep the service alive
    pending::<()>().await;

    Ok(())
}
