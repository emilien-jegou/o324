use crate::config::GitFileFormatType;
use crate::managers::config_manager::{ConfigManager, ConfigManagerParameters};
use crate::managers::document_storage_manager::{DocumentStorageManager, IDocumentStorageManager};
use crate::managers::file_format_manager::{
    FileFormatManager, IFileFormatManager, JsonFileFormat, TomlFileFormat, YamlFileFormat,
};
use crate::managers::git_manager::GitManager;
use crate::managers::git_sync_manager::GitSyncManager;
use crate::managers::metadata_document_manager::MetadataDocumentManager;
use crate::managers::task_document_manager::TaskDocumentManager;
use crate::models::metadata_document::MetadataDocument;
use crate::models::task_document::TaskDocument;
use crate::services::metadata_service::MetadataService;
use crate::services::task_service::TaskService;
use std::sync::Arc;

use super::config::GitStorageConfig;
use shaku::{module, HasComponent};

pub type TaskDocumentStorage = dyn IDocumentStorageManager<TaskDocument>;
pub type MetadataDocumentStorage = dyn IDocumentStorageManager<MetadataDocument>;

pub trait FileFormatModule: HasComponent<dyn IFileFormatManager> {}

macro_rules! create_format_module {
    ($module_name:ident, $file_format:ty) => {
        module! {
            $module_name: FileFormatModule {
                components = [ #[lazy] FileFormatManager<$file_format> ],
                providers = []
            }
        }
    };
}

create_format_module!(JsonFormatModule, JsonFileFormat);
create_format_module!(YamlFormatModule, YamlFileFormat);
create_format_module!(TomlFormatModule, TomlFileFormat);

module! {
    pub GitStorageModule {
        components = [
            ConfigManager,
            #[lazy] GitManager,
            #[lazy] GitSyncManager,
            #[lazy] MetadataDocumentManager,
            #[lazy] MetadataService,
            #[lazy] TaskDocumentManager,
            #[lazy] TaskService,
            #[lazy] DocumentStorageManager<MetadataDocument>,
            #[lazy] DocumentStorageManager<TaskDocument>,
        ],
        providers = [],


        use dyn FileFormatModule {
            components = [ dyn IFileFormatManager ],
            providers = []
        }
    }
}

pub fn build_from_config(config: &GitStorageConfig) -> eyre::Result<GitStorageModule> {
    let storage_path = config.get_git_storage_path()?;
    let git_storage_path = std::path::Path::new(&storage_path);

    let file_format_module: Arc<dyn FileFormatModule> = match config.get_file_format_type()? {
        GitFileFormatType::Json => Arc::new(JsonFormatModule::builder().build()),
        GitFileFormatType::Yaml => Arc::new(YamlFormatModule::builder().build()),
        GitFileFormatType::Toml => Arc::new(TomlFormatModule::builder().build()),
    };

    let module = GitStorageModule::builder(file_format_module)
        .with_component_parameters::<ConfigManager>(ConfigManagerParameters {
            repository_path: git_storage_path.to_path_buf(),
            remote_origin_url: config.git_remote_origin_url.clone(),
        })
        .build();

    Ok(module)
}
