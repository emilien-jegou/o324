use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct GitStorageConfig {
    /// path of git directory (default to ~/.local/share/3to4/git-storage-data)
    git_storage_path: Option<String>,
}

impl GitStorageConfig {
    pub fn get_git_storage_path(&self) -> eyre::Result<String> {
        let path_raw = self
            .git_storage_path
            .clone()
            .unwrap_or("~/.local/share/3to4/git-storage-data".to_owned());

        Ok(shellexpand::full(&path_raw)?.into_owned())
    }
}

