use std::sync::Arc;

use retry::delay::Fixed;
use shaku::{Component, Interface};

use crate::{git_actions, utils::files};

use super::{
    config_manager::IConfigManager, file_format_manager::IFileFormatManager,
    git_sync_manager::IGitSyncManager,
};

pub trait IGitManager: Interface {
    fn repository_is_initialized(&self) -> eyre::Result<()>;
    fn sync(&self) -> eyre::Result<()>;
    fn init_repository(&self) -> eyre::Result<()>;
    fn commit_on_change(&self) -> eyre::Result<()>;
}

#[derive(Component)]
#[shaku(interface = IGitManager)]
    pub struct GitManager {
    #[shaku(inject)]
    file_format_manager: Arc<dyn IFileFormatManager>,
    #[shaku(inject)]
    git_sync_manager: Arc<dyn IGitSyncManager>,
    #[shaku(inject)]
    config: Arc<dyn IConfigManager>,
}

impl IGitManager for GitManager {
    fn repository_is_initialized(&self) -> eyre::Result<()> {
        files::check_path_is_git_directory(&self.config.get_repository_path())
            .map_err(|e| eyre::eyre!("storage is not initialized, got error: {e}"))
    }

    fn sync(&self) -> eyre::Result<()> {
        let repository = git2::Repository::open(self.config.get_repository_path())?;
        retry::retry(Fixed::from_millis(100).take(3), || -> eyre::Result<()> {
            git_actions::fetch(&repository)?;
            self.git_sync_manager.rebase_with_auto_resolve()?;
            git_actions::push(&repository)?;
            Ok(())
        })
        .map_err(|e| eyre::eyre!("couldn't synchronize changes retried three times: {e}"))?;

        Ok(())
    }

    fn init_repository(&self) -> eyre::Result<()> {
        git_actions::init(
            &self.config.get_repository_path(),
            &self.config.get_remote_origin_url(),
        )?;
        Ok(())
    }

    fn commit_on_change(&self) -> eyre::Result<()> {
        let repository = git2::Repository::open(self.config.get_repository_path())?;
        let rg = format!("*\\.{}", self.file_format_manager.file_extension());
        git_actions::stage_and_commit_changes(&repository, "test", &[&rg])?;
        Ok(())
    }
}
