use crate::sync_runner::SyncRunner;
use crate::transaction::Transaction;
use crate::{connection_config::ConnectionConfig, document_parser::DocumentParser};
use crate::{git_actions, Client, QueryRunner, StoreError, StoreResult, SyncConflict};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Connection {
    pub(crate) name: String,
    pub(crate) document_parser: DocumentParser,
    pub(crate) repository_path: PathBuf,
    #[cfg(target_os = "linux")]
    pub(crate) repository: Arc<Mutex<git2::Repository>>,
}

// TODO: remove all eyre logic
// TODO: clean up error handling
impl Connection {
    pub fn initialize(config: ConnectionConfig) -> StoreResult<Connection> {
        #[cfg(target_os = "linux")]
        let repository = git_actions::init(&config.repository_path, &config.remote_origin_url)
            .map_err(StoreError::git_error)?;

        Ok(Self {
            name: config.connection_name.clone(),
            document_parser: config.document_parser,
            repository_path: config.repository_path,
            #[cfg(target_os = "linux")]
            repository: Arc::new(Mutex::new(repository)),
        })
    }

    // TODO: tests
    pub fn client(&self) -> Client {
        Client::new(self, QueryRunner::new(self))
    }

    // TODO: tests
    pub fn transaction(&self) -> Transaction {
        Transaction::new(self, QueryRunner::new(self))
    }

    // TODO: tests
    pub fn sync<F>(&self, callback: F) -> StoreResult<()>
    where
        F: FnMut(&QueryRunner<'_>, &mut Vec<SyncConflict>) -> eyre::Result<()>,
    {
        SyncRunner::try_new(self).sync(callback)
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::Document;
    use crate::{document_parser::JsonParser, IQueryRunner};
    use git_document_db_macros::Document;
    use lazy_regex::Regex;
    use std::collections::BTreeSet;
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
    ) -> eyre::Result<(TempDir, git2::Repository, git2::Repository, Connection)> {
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

        let config = ConnectionConfig::builder()
            .document_parser(JsonParser::get())
            .repository_path(repo_path.clone())
            .remote_origin_url(origin_url)
            .build();

        let connection = Connection::initialize(config).unwrap();

        let repo = git2::Repository::open(repo_path)?;

        Ok((temp, repo, origin, connection))
    }

    #[test]
    pub fn test_simple_client_initialization() {
        build_simple_client().unwrap();
    }

    #[test]
    pub fn test_simple_client_get() {
        let (_temp, _local, _origin, connection) = build_simple_client().unwrap();

        let test_document = connection.client().get::<TestDocument>("test").unwrap();
        assert!(test_document.is_none())
    }

    #[test]
    pub fn test_simple_client_get_and_save() {
        let (_temp, _local, _origin, connection) = build_simple_client().unwrap();
        let client = connection.client();

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
        let (_temp, _local, _origin, connection) = build_simple_client().unwrap();
        let client = connection.client();

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
