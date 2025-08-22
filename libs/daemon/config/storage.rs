use crate::{config::Config, core::storage::Storage, entities::MODELS};

pub fn create_storage_from_config(config: &Config) -> eyre::Result<Storage> {
    let profile_config = config.get_current_profile()?;

    let mut db_path = profile_config.get_storage_location().clone();
    std::fs::create_dir_all(&db_path)?;
    db_path.push("storage.db");
    let storage = Storage::try_new(&db_path, &MODELS)
        .map_err(|e| eyre::eyre!("Couldn't initialize storage on path {db_path:?}: {e}"))?;
    Ok(storage)
}
