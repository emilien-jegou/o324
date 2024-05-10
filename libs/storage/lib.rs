use inflector::cases::pascalcase::to_pascal_case;
use o324_config::ProfileConfig;
use o324_storage_core::StorageConfig;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

pub use o324_storage_core::{
    LockType, StorageClient, StorageContainer, Task, TaskAction, TaskBuilder, TaskId, TaskUpdate,
    TransactionContainer,
};

#[derive(EnumString, Display)]
pub enum BuiltinStorageType {
    #[cfg(feature = "git")]
    Git,
}

impl BuiltinStorageType {
    fn from_str_snakecase(s: &str) -> eyre::Result<Self> {
        Ok(Self::from_str(&to_pascal_case(s))?)
    }
}

#[derive(Deserialize)]
#[serde(bound = "S: DeserializeOwned")]
pub struct Config<S: StorageConfig> {
    /// name of this computer
    pub computer_name: String,

    /// default storage type to be used by frontends (default to: git)
    pub default_storage_type: Option<String>,
    pub storage: S,
}

pub fn load_builtin_storage_from_profile(
    profile: &ProfileConfig,
) -> eyre::Result<StorageContainer> {
    let storage_type = BuiltinStorageType::from_str_snakecase(&profile.storage_type)
        .map_err(|_| eyre::eyre!("Unsupported profile name"))?;

    match storage_type {
        #[cfg(feature = "git")]
        BuiltinStorageType::Git => {
            load_storage_from_value::<o324_storage_git::GitStorageConfig>(&profile.details)
        }
    }
}

pub fn load_storage_from_value<SC>(details: &toml::Value) -> eyre::Result<StorageContainer>
where
    SC: StorageConfig,
{
    let config: SC = details.clone().try_into()?;
    config.try_into_storage()
}
