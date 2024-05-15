use o324_core::Core;
use tokio::sync::{RwLock, RwLockReadGuard};

pub struct AppCore(RwLock<Core>);

impl AppCore {
    pub fn new(core: Core) -> Self {
        Self(RwLock::new(core))
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, Core> {
        self.0.read().await
    }

    pub async fn update(&self, new_core: Core) {
        let mut w = self.0.write().await;
        *w = new_core;
    }
}
