use git2::Repository;
use std::path::Path;

pub fn check_path_is_directory(path: &Path) -> eyre::Result<()> {
    if !path.exists() {
        return Err(eyre::eyre!("Directory {:?} doesn't exist", path));
    } else if !path.is_dir() {
        return Err(eyre::eyre!("Path {:?} is not a directory", path));
    }
    Ok(())
}

pub fn check_path_is_git_directory(path: &Path) -> eyre::Result<()> {
    check_path_is_directory(&path)?;
    match Repository::discover(path) {
        Ok(_) => Ok(()),
        Err(_) => Err(eyre::eyre!("Path {:?} is not a git directory", path)),
    }
}

/// Create directory and all necessary parent directories of a given path
pub fn create_dir_if_not_exists_deep(path: &Path) -> eyre::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

// Initialize a new repository at the specified path
pub fn init_git_repo_at_path(path: &Path) -> eyre::Result<()> {
    Repository::init(path)?;
    Ok(())
}

