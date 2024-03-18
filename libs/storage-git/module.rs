use crate::{managers::metadata_document_manager::MetadataDocumentManager, services::storage_sync_service::StorageSyncService};
use crate::managers::task_document_manager::TaskDocumentManager;
use crate::services::metadata_service::MetadataService;
use crate::services::task_service::TaskService;
use teloc::{inject, Dependency, Resolver, ServiceProvider};

use super::config::GitStorageConfig;

#[derive(Dependency)]
pub struct Module {
    // Services
    pub metadata_service: MetadataService,
    pub task_service: TaskService,
    pub storage_sync_service: StorageSyncService,
    pub git_service: GitService,

    // Managers
    pub metadata_document_manager: MetadataDocumentManager,
    pub task_document_manager: MetadataDocumentManager,
}

#[derive(Clone)]
pub struct GitService(pub git_document_db::Client);

#[inject]
impl GitService {
    pub fn new(git_document_db: &git_document_db::Client) -> Self {
        Self(git_document_db.clone())
    }
}

pub fn build_from_config(config: &GitStorageConfig) -> eyre::Result<Module> {
    let storage_path = config.get_git_storage_path()?;
    let git_storage_path = std::path::Path::new(&storage_path);

    let config = git_document_db::ClientConfig::builder()
        .document_parser(git_document_db::document_parser::JsonParser::get())
        .repository_path(git_storage_path.to_path_buf())
        .remote_origin_url(config.git_remote_origin_url.to_string())
        .build();

    let client = git_document_db::Client::initialize(config)?;

    let sp = ServiceProvider::new()
        .add_transient::<GitService>()
        .add_transient::<MetadataDocumentManager>()
        .add_transient::<MetadataService>()
        .add_transient::<StorageSyncService>()
        .add_transient::<TaskDocumentManager>()
        .add_transient::<TaskService>()
        .add_transient::<Module>()
        .add_instance(&client);

    let module: Module = sp.resolve();
    Ok(module)
}
