use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf; // <-- Import PathBuf

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub core: CoreConfig,
    pub profile: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    /// Name of this computer
    pub computer_name: String,

    /// Profile used by default when none are specified
    pub default_profile_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileConfig {
    /// Where the o324 database will be located (default: ~/.local/share/o324/)
    storage_location: Option<String>,

    ///// Desired synchronization method (e.g. git)
    //pub storage_sync_type: Option<String>,

    // Rest of the storage config as a flexible structure
    #[serde(flatten)]
    pub details: toml::Value,
}

impl Config {
    /// Gets the current profile based on the `default_profile_name` in the core configuration.
    pub fn get_current_profile(&self) -> eyre::Result<&ProfileConfig> {
        let profile_name = self.core.get_default_profile_name();
        self.profile
            .get(&profile_name)
            .ok_or_else(|| eyre::eyre!("Profile '{profile_name}' not found in config"))
    }
}

impl ProfileConfig {
    /// Gets the storage location for this profile.
    pub fn get_storage_location(&self) -> PathBuf {
        let path_str = self
            .storage_location
            .as_deref()
            .unwrap_or("~/.local/share/o324");
        let expanded_path = shellexpand::tilde(path_str);
        PathBuf::from(expanded_path.as_ref())
    }
}

impl CoreConfig {
    /// Return the default profile name is set or "default"
    pub fn get_default_profile_name(&self) -> String {
        self.default_profile_name
            .clone()
            .unwrap_or("default".to_owned())
    }
}
