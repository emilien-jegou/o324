use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct AppConfigInner {
    pub profile_name: Option<String>,
    pub config_path: String,
}

pub struct AppConfig(RwLock<AppConfigInner>);

impl AppConfig {
    pub fn new(inner: AppConfigInner) -> Self {
        Self(RwLock::new(inner))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, AppConfigInner> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, AppConfigInner> {
        self.0.write().await
    }
}
