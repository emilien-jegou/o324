use shaku::module;

use crate::managers::git_manager::{GitManager, GitManagerParameters};
use crate::managers::metadata_manager::{MetadataManager, MetadataManagerParameters};
use crate::managers::model_manager::{
    IModelManager, JsonModel, ModelManager, ModelManagerParameters,
};
use crate::managers::task_manager::{TaskManager, TaskManagerParameters};
use crate::models::metadata_document::MetadataDocument;
use crate::models::task_document::TaskDocument;
use crate::providers::git_transaction_provider::GitTransaction;

use super::config::GitStorageConfig;

pub type TaskModel = dyn IModelManager<TaskDocument>;
pub type MetadataModel = dyn IModelManager<MetadataDocument>;

module! {
    pub GitStorageModule {
        components = [
            GitManager,
            MetadataManager,
            TaskManager,
            ModelManager<JsonModel, TaskDocument>,
            ModelManager<JsonModel, MetadataDocument>,
        ],
        providers = [GitTransaction]
    }
}

pub fn build_from_config(config: &GitStorageConfig) -> eyre::Result<GitStorageModule> {
    let storage_path = config.get_git_storage_path()?;
    let git_storage_path = std::path::Path::new(&storage_path);

    let module = GitStorageModule::builder()
        .with_component_parameters::<GitManager>(GitManagerParameters {
            repository_path: git_storage_path.to_path_buf(),
            remote_origin_url: config.git_remote_origin_url.clone(),
        })
        .with_component_parameters::<MetadataManager>(MetadataManagerParameters {
            git_storage_path: git_storage_path.to_path_buf(),
            metadata_path: git_storage_path.join("__metadata.json"),
        })
        .with_component_parameters::<TaskManager>(TaskManagerParameters {
            repository_path: git_storage_path.to_path_buf(),
        })
        .with_component_parameters::<ModelManager<JsonModel, TaskDocument>>(
            ModelManagerParameters {
                base_path: git_storage_path.to_path_buf(),
            },
        )
        .with_component_parameters::<ModelManager<JsonModel, MetadataDocument>>(
            ModelManagerParameters {
                base_path: git_storage_path.to_path_buf(),
            },
        )
        .build();

    Ok(module)
}
