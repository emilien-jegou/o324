use std::path::Path;

use crate::Config;

pub fn load(config_path: &str) -> eyre::Result<Config> {
    let content = read_file_content_if_exist(config_path)?
        .ok_or_else(|| eyre::eyre!("config path '{config_path}' was not found"))?;

    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

fn read_file_content_if_exist(file_path: &str) -> eyre::Result<Option<String>> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    Ok(Some(content))
}
