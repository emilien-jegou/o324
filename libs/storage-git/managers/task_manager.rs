use shaku::{Component, Interface};
use std::{path::PathBuf, sync::Arc};

use super::metadata_manager::IMetadataManager;

pub trait ITaskManager: Interface {}

#[derive(Component)]
#[shaku(interface = ITaskManager)]
pub struct TaskManager {
    #[shaku(inject)]
    metadata_manager: Arc<dyn IMetadataManager>,
    repository_path: PathBuf,
}

impl ITaskManager for TaskManager {}
