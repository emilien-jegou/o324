use async_std::fs::File;
use async_std::prelude::*;
use o324_storage::StorageConfig;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(bound = "S: DeserializeOwned")]
pub struct Config<S: StorageConfig> {
    /// default storage type to be used by frontends (default to: git)
    pub default_storage_type: Option<String>,
    pub storage: S,
}

async fn read_file_content(file_path: &str) -> eyre::Result<Option<String>> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Ok(None);
    }

    let mut file = File::open(path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    Ok(Some(content))
}

pub fn get_default_storage_config<S>() -> Config<S>
where
    S: StorageConfig,
{
    Config {
        // TODO: fix this:
        default_storage_type: Some("git".to_string()),
        storage: S::default(),
    }
}

pub async fn get_config_from_path<S>(config_path: &str) -> eyre::Result<Config<S>>
where
    S: StorageConfig,
{
    let content = read_file_content(config_path)
        .await?
        .ok_or_else(|| eyre::eyre!("config path '{config_path}' was not found"))?;

    let config: Config<S> = toml::from_str(&content)?;

    Ok(config)
}
