use super::{config::GitStorageConfig, transaction::GitTransaction};
use crate::{utils::files, PinFuture, Storage, StorageBox, StorageConfig, Task, TransactionBox};

/// Save data as json inside of a git directory
pub struct GitStorage {
    config: GitStorageConfig,
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

    fn create_task(&self, task: Task) -> PinFuture<eyre::Result<()>> {
        Box::pin(async move {
            let now = chrono::Local::now();
            let formatted_date = now.format("%Y-%m-%d.json").to_string();
            let storage_path = self.config.get_git_storage_path()?;
            let path = std::path::Path::new(&storage_path);
            let full_path = path.join(formatted_date);

            let mut data: Vec<Task> = files::read_json_document_as_struct_with_default(&full_path)?;

            data.push(task);

            files::save_json_document(&full_path, &data)?;
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
