use std::{io::Write, path::Path};
use super::Config;

pub fn load(config_path: &str) -> eyre::Result<Config> {
    let content = read_file_content_if_exist(config_path)?
        .ok_or_else(|| eyre::eyre!("Failed to read config file: {config_path}"))?;

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

#[allow(dead_code)]
pub fn save(config_path: &str, config: &Config) -> eyre::Result<()> {
    let toml_string =
        toml::to_string(config).map_err(|e| eyre::eyre!("Failed to serialize config: {e}"))?;

    let path = Path::new(config_path);
    let mut file = std::fs::File::create(path)
        .map_err(|e| eyre::eyre!("Failed to create or truncate file '{config_path}': {e}"))?;

    file.write_all(toml_string.as_bytes())
        .map_err(|e| eyre::eyre!("Failed to write to file '{config_path}': {e}"))?;

    Ok(())
}
