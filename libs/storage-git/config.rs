use inflector::cases::pascalcase::to_pascal_case;
use serde::Serialize;
use serde_derive::Deserialize;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display)]
pub enum GitFileFormatType {
    Json,
    Yaml,
    Toml,
}

impl GitFileFormatType {
    fn try_from_str_lowercase(s: &str) -> eyre::Result<Self> {
        Self::from_str(&to_pascal_case(s)).map_err(|_| {
            eyre::eyre!(
                "Invalid file format type specified, please select one of `toml`, `yaml`, `json`"
            )
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GitStorageConfig {
    /// Name of the connection, will appear in commit history
    pub connection_name: Option<String>,

    /// Path of git directory where tasks are stored (default to ~/.local/share/o324/git-storage-data)
    pub git_storage_path: Option<String>,

    /// Path of the remote directory where tasks should be persisted
    pub git_remote_origin_url: String,

    /// Storage format of the files (default to: json)
    pub git_file_format_type: Option<String>,
}

impl GitStorageConfig {
    pub fn get_connection_name(&self) -> String {
        self.connection_name
            .clone()
            .unwrap_or_else(|| String::from("o324"))
    }

    pub fn get_file_format_type(&self) -> eyre::Result<GitFileFormatType> {
        self.git_file_format_type
            .as_ref()
            .map(|s| GitFileFormatType::try_from_str_lowercase(s))
            .unwrap_or(Ok(GitFileFormatType::Json))
    }

    pub fn get_git_storage_path(&self) -> eyre::Result<String> {
        let path_raw = self
            .git_storage_path
            .clone()
            .unwrap_or("~/.local/share/o324/git-storage-data".to_owned());

        Ok(shellexpand::full(&path_raw)?.into_owned())
    }
}
