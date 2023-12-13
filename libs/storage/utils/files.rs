use git2::Repository;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub fn check_path_is_directory(path: &Path) -> eyre::Result<()> {
    if !path.exists() {
        return Err(eyre::eyre!("Directory {:?} doesn't exist", path));
    } else if !path.is_dir() {
        return Err(eyre::eyre!("Path {:?} is not a directory", path));
    }
    Ok(())
}

pub fn check_path_is_git_directory(path: &Path) -> eyre::Result<()> {
    check_path_is_directory(path)?;
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

pub fn read_json_document_as_struct_with_default<
    T: DeserializeOwned + Default + 'static,
    P: AsRef<Path>,
>(
    path: P,
) -> eyre::Result<T> {
    let path = path.as_ref();
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(T::default())
    }
}

pub fn save_json_document<T: Serialize, P: AsRef<Path>>(path: P, data: &T) -> eyre::Result<()> {
    let serialized = serde_json::to_string(data)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}
