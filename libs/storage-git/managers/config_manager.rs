use shaku::{Component, Interface};
use std::path::PathBuf;

pub trait IConfigManager: Interface {
    fn get_repository_path(&self) -> PathBuf;
    fn get_remote_origin_url(&self) -> String;
}

#[derive(Component)]
#[shaku(interface = IConfigManager)]
pub struct ConfigManager {
    repository_path: PathBuf,
    remote_origin_url: String,
}

impl IConfigManager for ConfigManager {
    fn get_repository_path(&self) -> PathBuf {
        self.repository_path.clone()
    }

    fn get_remote_origin_url(&self) -> String {
        self.remote_origin_url.clone()
    }
}
