use shaku::module;

use crate::managers::git_manager::{GitManager, GitManagerParameters};
use crate::managers::metadata_manager::{MetadataManager, MetadataManagerParameters};
use crate::providers::git_transaction_provider::GitTransaction;

use super::config::GitStorageConfig;

module! {
    pub(crate) GitStorageModule {
        components = [GitManager, MetadataManager],
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
        .build();

    Ok(module)
}
