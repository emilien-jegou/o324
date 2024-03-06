use std::{path::PathBuf, sync::Arc};

use retry::delay::Fixed;
use shaku::{Component, Interface};

use crate::{git_actions, git_synchronize, utils::files};

use super::metadata_manager::IMetadataManager;

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
    metadata_manager: Arc<dyn IMetadataManager>,
    repository_path: PathBuf,
    remote_origin_url: String,
}

impl IGitManager for GitManager {
    fn repository_is_initialized(&self) -> eyre::Result<()> {
        files::check_path_is_git_directory(&self.repository_path)
            .map_err(|e| eyre::eyre!("storage is not initialized, got error: {e}"))
    }

    fn sync(&self) -> eyre::Result<()> {
        let repository = git2::Repository::open(&self.repository_path)?;
        retry::retry(Fixed::from_millis(100).take(3), || -> eyre::Result<()> {
            git_actions::fetch(&repository)?;
            git_synchronize::rebase_with_auto_resolve(&self.metadata_manager, &repository)?;
            git_actions::push(&repository)?;
            Ok(())
        })
        .map_err(|e| eyre::eyre!("couldn't synchronize changes retried three times: {e}"))?;

        Ok(())
    }

    fn init_repository(&self) -> eyre::Result<()> {
        git_actions::init(&self.repository_path, &self.remote_origin_url)?;
        Ok(())
    }

    fn commit_on_change(&self) -> eyre::Result<()> {
        let repository = git2::Repository::open(&self.repository_path)?;
        git_actions::stage_and_commit_changes(&repository, "test", &["*\\.json"])?;
        Ok(())
    }
}
