use crate::{config, Core};
use o324_storage::{
    storage::{git::GitStorageConfig, in_memory::InMemoryStorageConfig},
    BuiltinStorageType, StorageConfig,
};

pub async fn load(storage_type: BuiltinStorageType, config_path: &str) -> eyre::Result<Core> {
    match storage_type {
        BuiltinStorageType::Git => load_core::<GitStorageConfig>(config_path).await,
        BuiltinStorageType::InMemory => load_core::<InMemoryStorageConfig>(config_path).await,
    }
}

pub async fn load_core<SC>(config_path: &str) -> eyre::Result<Core>
where
    SC: StorageConfig,
{
    // TODO: we want the default_storage_type option to be effective
    // right now it doesn't produce anything.
    let mut found_config_file = Ok(());
    let config = match config::get_config_from_path::<SC>(config_path).await {
        Ok(v) => v,
        Err(e) => {
            found_config_file = Err(e);
            config::get_default_storage_config::<SC>()
        }
    };

    let storage = config.storage.to_storage();

    Ok(Core {
        storage,
        found_config_file,
    })
}
