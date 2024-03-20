use std::sync::Arc;

use crate::{providers::git_transaction_provider::GitTransactionService, services::git_service::GitService};
use crate::services::metadata_service::MetadataService;
use crate::services::storage_sync_service::StorageSyncService;
use crate::services::task_service::TaskService;
use teloc::{Dependency, Resolver, ServiceProvider};

use super::config::GitStorageConfig;

#[derive(Dependency)]
pub struct Module {
    // Services
    pub metadata_service: MetadataService,
    pub task_service: TaskService,
    pub storage_sync_service: StorageSyncService,
    pub git_service: GitService,
    pub git_transaction_service: GitTransactionService,
}

pub fn build_from_config(config: &GitStorageConfig) -> eyre::Result<Module> {
    let storage_path = config.get_git_storage_path()?;
    let git_storage_path = std::path::Path::new(&storage_path);

    let config = git_document_db::ConnectionConfig::builder()
        .connection_name(config.get_connection_name())
        .document_parser(git_document_db::document_parser::JsonParser::get())
        .repository_path(git_storage_path.to_path_buf())
        .remote_origin_url(config.git_remote_origin_url.to_string())
        .build();

    let connection = git_document_db::Connection::initialize(config)?;

    let sp = ServiceProvider::new()
        .add_transient::<MetadataService>()
        .add_transient::<StorageSyncService>()
        .add_transient::<TaskService>()
        .add_transient::<GitService>()
        .add_transient::<GitTransactionService>()
        .add_transient::<Module>()
        .add_instance(Arc::new(connection));

    let module: Module = sp.resolve();
    Ok(module)
}
