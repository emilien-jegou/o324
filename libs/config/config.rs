use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfig {
    /// Name of this computer
    pub computer_name: String,

    /// Profile used by default when none are specified
    pub default_profile_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProfileConfig {
    /// Type of storage (e.g. git)
    pub storage_type: String,

    // Rest of the storage config as a flexible structure
    #[serde(flatten)]
    pub details: toml::Value,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub core: CoreConfig,
    pub profile: HashMap<String, ProfileConfig>,
}

impl CoreConfig {
    /// Return the default profile name is set or "default"
    pub fn get_default_profile_name(&self) -> String {
        self.default_profile_name
            .clone()
            .unwrap_or("default".to_owned())
    }
}
