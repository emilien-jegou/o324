use std::{fs::File, io::{Read, Write}, path::Path};

use crate::{
    utils::{files, semaphore::Semaphore},
    PinFuture, Storage, StorageBox, StorageConfig, Task, Transaction, TransactionBox,
};
use serde_derive::Deserialize;

type GitDailyDocument = Vec<Task>;

fn read_document_from_path<P: AsRef<Path>>(path: P) -> eyre::Result<GitDailyDocument> {
    let path = path.as_ref();
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(GitDailyDocument::default())
    }
}

fn save_document_from_path<P: AsRef<Path>>(path: P, data: &GitDailyDocument) -> eyre::Result<()> {
    let serialized = serde_json::to_string(data)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}


/// Save data as json inside of a git directory
pub struct GitStorage {
    config: GitStorageConfig,
}

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

// - get the path of the storage, default to .local/share/3to4/git-storage-data
// - check if the directory if not, exit, the user has to call ./3to4 init to init the storage with
// the given config
// - try get file for the current day, file have format [yyyy]-[mm]-[dd].json
//    - if doesnt exist -> return false
//    - if exist pass it to git data parser, check if any task as not been ended

impl StorageConfig for GitStorageConfig {
    type Storage = GitStorage;

    fn to_storage(self) -> StorageBox {
        StorageBox::new(GitStorage::new(self))
    }
}

// NOTE: we may only want to expose the try_lock as we want all action to be executed in a
// transaction, methods such as has_active_task may be better suited for the Transaction struct
// instead

impl Storage for GitStorage {
    fn debug_message(&self) {
        println!("Git storage");
        println!("config: {:?}", self.config);
    }

    fn init(&self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let storage_path = self.config.get_git_storage_path()?;
            let path = std::path::Path::new(&storage_path);
            files::create_dir_if_not_exists_deep(&path)?;
            files::init_git_repo_at_path(&path)?;
            println!("Initialized git directory on: {storage_path}");
            Ok(())
        })
    }

    fn try_lock(&self) -> PinFuture<eyre::Result<TransactionBox>> {
        Box::pin(async move {
            // We want to verify that the repository is a git directory before running the
            // transaction, in case it's not the user will have to run the 'init' command
            self.check_is_init().await?;
            Ok(TransactionBox::new(GitTransaction::try_new()?))
        })
    }

    fn has_active_task(&self) -> PinFuture<eyre::Result<bool>> {
        Box::pin(async move { Ok(false) })
    }

    fn start_new_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let now = chrono::Local::now();
            let formatted_date = now.format("%Y-%m-%d.json").to_string();
            let storage_path = self.config.get_git_storage_path()?;
            let path = std::path::Path::new(&storage_path);
            let full_path = path.join(formatted_date);

            let mut data = read_document_from_path(&full_path)?;

            data.push(task);

            save_document_from_path(&full_path, &data)?;
            Ok(())
        })
    }
}

impl GitStorage {
    pub fn new(config: GitStorageConfig) -> Self {
        GitStorage { config }
    }

    async fn check_is_init(&self) -> eyre::Result<()> {
        let storage_path = self.config.get_git_storage_path()?;
        let path = std::path::Path::new(&storage_path);
        files::check_path_is_git_directory(&path)
            .map_err(|e| eyre::eyre!("storage is not initialized, got error: {e}"))
    }
}

pub struct GitTransaction {
    lock: Semaphore,
}

const GIT_SEMAPHORE_NAME: &str = "3to4-git-transaction-3";

impl GitTransaction {
    pub fn try_new() -> eyre::Result<Self> {
        let mut lock = Semaphore::try_new(GIT_SEMAPHORE_NAME)?;
        lock.try_acquire()?;
        Ok(GitTransaction { lock })
    }
}

impl Transaction for GitTransaction {
    fn release(&mut self) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            self.lock.release()?;
            Ok(())
        })
    }
}

impl Drop for GitTransaction {
    fn drop(&mut self) {
        // TODO: this is somewhat unsafe
        self.lock.release().expect("Couldn't release semaphore");
    }
}
