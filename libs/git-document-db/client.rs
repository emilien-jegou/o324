use crate::git_actions::rebase;
use crate::utils::files;
use crate::{client_config::ClientConfig, document_parser::DocumentParser};
use crate::{git_actions, Document, StoreError, StoreResult};
use lazy_regex::Regex;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Client {
    document_parser: DocumentParser,
    repository_path: PathBuf,
    remote_origin_url: String,
    repository: Arc<Mutex<git2::Repository>>,
}

// TODO: remove all eyre logic
// TODO: clean up error handling

impl Client {
    pub fn initialize(config: ClientConfig) -> StoreResult<Client> {
        let repository = git_actions::init(&config.repository_path, &config.remote_origin_url)
            .map_err(|e| StoreError::GitError(e.to_string()))?;
        Ok(Self {
            document_parser: config.document_parser,
            repository_path: config.repository_path,
            remote_origin_url: config.remote_origin_url,
            repository: Arc::new(Mutex::new(repository)),
        })
    }

    pub fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>> {
        let mut path = self.repository_path.join(document_id);

        files::add_file_extension(&mut path, self.document_parser.file_extension());

        if path.exists() {
            let contents = std::fs::read_to_string(path).map_err(StoreError::IoError)?;
            let data = self
                .document_parser
                .deserialize(&contents)
                .map_err(|e| StoreError::CorruptedDocument(e.to_string()))?;
            let mut document: T = serde_json::from_value(data)
                .map_err(|e| StoreError::DocumentParseError(e.to_string()))?;
            document.set_document_id(document_id);
            Ok(Some(document))
        } else {
            Ok(None)
        }
    }

    pub fn get_document_list(&self) -> StoreResult<Vec<String>> {
        let extension = self.document_parser.file_extension();

        // Get list of documents
        let matching_files = files::find_matching_files(
            &self.repository_path,
            &Regex::new(&format!(".*{extension}$"))
                .map_err(|e| StoreError::OperationFailed(e.to_string()))?,
        )
        .map_err(|e| StoreError::OperationFailed(e.to_string()))?;

        Ok(matching_files
            .into_iter()
            .map(|file| {
                let path = std::path::Path::new(&file);
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect())
    }

    pub fn find_matching<T: Document>(
        &self,
        document_id_regex: &lazy_regex::Regex,
    ) -> StoreResult<Vec<T>> {
        let document_list = self.get_document_list()?;

        Ok(document_list
            .into_iter()
            .filter(|d| document_id_regex.is_match(d))
            .map(|document_id| self.get::<T>(&document_id))
            .collect::<StoreResult<Vec<Option<T>>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<T>>())
    }

    pub fn save<T: Document>(&self, document: &T) -> StoreResult<()> {
        let mut path = self.repository_path.join(document.get_document_id());

        files::add_file_extension(&mut path, self.document_parser.file_extension());
        let serialized = self
            .document_parser
            .serialize(document)
            .map_err(|e| StoreError::CorruptedDocument(e.to_string()))?;
        std::fs::write(path, serialized.as_bytes()).map_err(StoreError::IoError)?;
        Ok(())
    }

    pub fn commit_on_change(&self) -> eyre::Result<()> {
        let repository = self
            .repository
            .lock()
            .map_err(|e| StoreError::OperationFailed(e.to_string()))?;
        let rg = format!("*\\.{}", self.document_parser.file_extension());
        git_actions::stage_and_commit_changes(&repository, "test", &[&rg])?;
        Ok(())
    }

    //global_db_lock
    pub fn sync<F>(&self, callback: F) -> StoreResult<()>
    where
        F: FnMut(&mut Vec<SyncConflict>) -> eyre::Result<()>,
    {
        let repository = self
            .repository
            .lock()
            .map_err(|e| StoreError::OperationFailed(e.to_string()))?;
        git_actions::fetch(&repository).map_err(|e| StoreError::OperationFailed(e.to_string()))?;
        let mut rebase = git_actions::rebase_current_branch(&repository)
            .map_err(|e| StoreError::OperationFailed(e.to_string()))?;

        match self.rebase_action(&mut rebase, callback) {
            Ok(()) => Ok(rebase
                .finalize()
                .map_err(|e| StoreError::OperationFailed(e.to_string()))?),
            Err(e) => {
                rebase
                    .abort()
                    .map_err(|e| StoreError::OperationFailed(e.to_string()))?;
                Err(e)
            }
        }
        .map_err(|e| StoreError::OperationFailed(e.to_string()))?;

        git_actions::push(&repository).map_err(|e| StoreError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    fn rebase_action<'a, F>(
        &'a self,
        rebase: &mut rebase::Rebase<'_>,
        mut callback: F,
    ) -> eyre::Result<()>
    where
        F: FnMut(&mut Vec<SyncConflict>) -> eyre::Result<()>,
    {
        let extension = self.document_parser.file_extension();
        let document_regex = &Regex::new(&format!(".*{extension}$"))
            .map_err(|e| StoreError::OperationFailed(e.to_string()))?;

        for op in rebase.iter() {
            let mut operation = op?;
            let conflict = operation.get_conflict()?;

            let document_conflicts = conflict
                .files
                .iter()
                .filter(|file| document_regex.is_match(&file.relative_file_path));

            let mut documents = document_conflicts
                .map(|file| {
                    let path = std::path::Path::new(&file.relative_file_path);
                    let id = path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("")
                        .to_string();

                    let (left, right) = if conflict.our_commit.timestamp < conflict.their_commit.timestamp {
                        (&file.our, &file.their)
                    } else {
                        (&file.their, &file.our)
                    };


                    Ok(SyncConflict::<'a> {
                        context: self,
                        id,
                        previous: match &file.previous {
                            Some(content) => {
                                Some(DocumentRef(self.document_parser.deserialize(content)?))
                            }
                            None => None,
                        },
                        left: DocumentRef(self.document_parser.deserialize(left)?),
                        right: DocumentRef(self.document_parser.deserialize(right)?),
                    })
                })
                .collect::<eyre::Result<Vec<_>>>()?;

            callback(&mut documents)?;

            conflict.stage_all()?;
            operation.commit_changes()?;
        }
        Ok(())
    }
}

pub struct DocumentRef(serde_json::Value);

impl DocumentRef {
    pub fn to_document<T: Document>(&self) -> eyre::Result<T> {
        let v: T = serde_json::from_value(self.0.clone())?;
        Ok(v)
    }
}

pub struct SyncConflict<'a> {
    context: &'a Client,
    pub id: String,
    pub previous: Option<DocumentRef>,
    pub left: DocumentRef,
    pub right: DocumentRef,
}

impl<'a> SyncConflict<'a> {
    pub fn save(&self, document: impl Document) -> eyre::Result<()> {
        self.context.save(&document)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::document_parser::JsonParser;

    use super::*;
    use git_document_db_macros::Document;
    use sugars::btset;
    use tempfile::{tempdir, TempDir};

    #[derive(PartialEq, Debug, Document)]
    struct TestDocument {
        #[document(id)]
        pub id: String,
        pub name: String,
    }

    #[test]
    pub fn test_document_id() {
        let mut doc = TestDocument {
            id: "my-id".to_string(),
            name: "Hello".to_string(),
        };

        assert_eq!(doc.get_document_id(), "my-id".to_string());
        doc.set_document_id("my-id-2");
        assert_eq!(doc.get_document_id(), "my-id-2".to_string());
    }

    #[test]
    pub fn test_serialize() {
        let doc = TestDocument {
            id: "omitted".to_string(),
            name: "Hello".to_string(),
        };
        let data: String = serde_json::to_string(&doc).unwrap();
        assert_eq!(data, r#"{"name":"Hello"}"#.to_string());
    }

    #[test]
    pub fn test_deserialize() {
        let data: TestDocument = serde_json::from_str(r#"{ "name": "Hello" }"#).unwrap();
        assert_eq!(data.id, "".to_string());
        assert_eq!(data.name, "Hello".to_string());
    }

    pub fn build_simple_client(
    ) -> eyre::Result<(TempDir, git2::Repository, git2::Repository, Client)> {
        let temp = tempdir().unwrap();
        let repo_path = temp.path().join("local_1");
        let origin_repo_path = temp.path().join("origin");
        let origin = git2::Repository::init_bare(&origin_repo_path)?;
        let origin_url = format!(
            "file://{}",
            origin_repo_path
                .to_str()
                .ok_or_else(|| eyre::eyre!("folder must use valid unicode characters"))
                .unwrap()
        );

        let config = ClientConfig::builder()
            .document_parser(JsonParser::get())
            .repository_path(repo_path.clone())
            .remote_origin_url(origin_url)
            .build();

        let client = Client::initialize(config).unwrap();

        let repo = git2::Repository::open(repo_path)?;

        Ok((temp, repo, origin, client))
    }

    #[test]
    pub fn test_simple_client_initialization() {
        build_simple_client().unwrap();
    }

    #[test]
    pub fn test_simple_client_get() {
        let (_temp, _local, _origin, client) = build_simple_client().unwrap();

        let test_document = client.get::<TestDocument>("test").unwrap();
        assert!(test_document.is_none())
    }

    #[test]
    pub fn test_simple_client_get_and_save() {
        let (_temp, _local, _origin, client) = build_simple_client().unwrap();

        let document = TestDocument {
            id: "test".to_string(),
            name: "hello".to_string(),
        };

        client.save(&document).unwrap();

        let expected_document = client.get::<TestDocument>("test").unwrap().unwrap();

        assert_eq!(expected_document, document);
    }

    #[test]
    pub fn test_simple_client_find_matching() {
        let (_temp, _local, _origin, client) = build_simple_client().unwrap();

        let with_id = |id: &str| TestDocument {
            id: id.to_string(),
            name: "hello".to_string(),
        };

        let save_with_id = |id: &str| client.save(&with_id(id)).unwrap();

        save_with_id("test");
        save_with_id("test--");
        save_with_id("--test");
        save_with_id("tes");
        save_with_id("t-e-s-t");
        save_with_id("--test--");

        let re = Regex::new("-*test").unwrap();
        let expected_document_1 = client.find_matching::<TestDocument>(&re).unwrap();

        assert_eq!(
            expected_document_1
                .iter()
                .map(|t| t.id.as_str())
                .collect::<BTreeSet<&str>>(),
            btset!["--test--", "test--", "test", "--test", "--test",]
        );

        // Same test with end of expression delimiter
        let re = Regex::new("^-*test$").unwrap();
        let expected_document_2 = client.find_matching::<TestDocument>(&re).unwrap();
        assert_eq!(
            expected_document_2
                .iter()
                .map(|t| t.id.as_str())
                .collect::<BTreeSet<&str>>(),
            btset!["test", "--test", "--test",]
        );
    }
}
