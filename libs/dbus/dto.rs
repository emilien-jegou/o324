use dyn_variant_macro::dyn_variant;
use serde::{Deserialize, Serialize};
use zvariant::{Optional, Type};

type TaskId = String;

#[derive(Type, Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct TaskDto {
    pub id: TaskId,
    pub id_prefix: String,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub computer_name: String,
    pub start: u64,
    pub end: Option<u64>,
    pub __hash: u64,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct TaskUpdateDto {
    pub task_name: Optional<String>,
    pub project: Optional<Option<String>>,
    pub tags: Optional<Vec<String>>,
    pub start: Optional<u64>,
    pub end: Optional<Option<u64>>,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct StartTaskInputDto {
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Type, Debug, Deserialize, Serialize)]
pub enum TaskActionType {
    Upsert,
    Delete,
}

#[derive(Type, Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct TaskActionUpsertDto {
    pub id: TaskId,
    pub task_name: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub computer_name: String,
    pub start: u64,
    pub end: Option<u64>,
    pub __hash: u64,
}

#[dyn_variant]
#[derive(Debug)]
pub enum TaskActionDto {
    Upsert(TaskActionUpsertDto),
    Delete(TaskId),
}

/// Defines the *type* of database operation to perform. This is a simple enum.
#[derive(Type, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum DbOperationTypeDto {
    ListTables,
    ScanTable,
}

/// The actual operation payload sent to the D-Bus service.
/// Contains the operation type and optional data for that operation.
#[derive(Type, Serialize, Deserialize, Debug)]
pub struct DbOperationDto {
    pub operation_type: DbOperationTypeDto,
    /// Only used for `ScanTable` operation.
    pub table_name: Option<String>,
}

#[dyn_variant]
#[derive(Debug)]
pub enum DbResultDto {
    TableList(Vec<String>),
    TableRows(Vec<String>),
}

#[dyn_variant]
#[derive(Debug)]
pub enum TaskByPrefixDto {
    /// Exact match was found
    Single(TaskDto),
    /// Multiple tasks were found
    Many(Vec<TaskDto>),
    /// No task found
    NotFound,
}

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct ActivityDto {
    pub id: String,
    pub app_title: Option<String>,
    pub app_name: String,
    pub at: u64,
    pub computer_name: String,
}
