use o324_storage::{
    storage::{git::GitStorageConfig, in_memory::InMemoryStorageConfig},
    BuiltinStorageType, StorageConfig, StorageBox,
};

pub mod config;

pub struct Core {
    storage: StorageBox,
    /// Ok - found | Err - not found with error reason
    found_config_file: Result<(), eyre::Error>,
}

impl Core {
    /// You should avoid calling this function
    pub fn get_inner_storage<'a>(&'a self) -> &'a StorageBox {
        &self.storage
    }

    pub fn has_found_config_file<'a>(&'a self) -> &'a Result<(), eyre::Error> {
        &self.found_config_file
    }
}

pub async fn load_core<SC>(config_path: &str) -> eyre::Result<Core>
where
    SC: StorageConfig,
{
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

pub async fn load(storage_type: BuiltinStorageType, config_path: &str) -> eyre::Result<Core> {
    match storage_type {
        BuiltinStorageType::Git => load_core::<GitStorageConfig>(&config_path).await,
        BuiltinStorageType::InMemory => load_core::<InMemoryStorageConfig>(&config_path).await,
    }
}
