use crate::git_actions::rebase;
use crate::query_runner::IQueryRunner;
use crate::{git_actions, Connection, Document, QueryRunner, StoreError, StoreResult};
use lazy_regex::Regex;

#[derive(Clone)]
pub struct SyncRunner<'a> {
    pub(crate) connection: &'a Connection,
    pub(crate) query_runner: QueryRunner<'a>,
}

impl<'a> SyncRunner<'a> {
    pub fn try_new(connection: &'a Connection) -> Self {
        let query_runner = QueryRunner { connection };
        Self {
            connection,
            query_runner,
        }
    }

    fn rebase_action<F>(&self, rebase: &mut rebase::Rebase<'_>, mut callback: F) -> eyre::Result<()>
    where
        F: FnMut(&QueryRunner<'_>, &mut Vec<SyncConflict>) -> eyre::Result<()>,
    {
        let extension = self.connection.document_parser.file_extension();
        let document_regex =
            &Regex::new(&format!(".*{extension}$")).map_err(StoreError::operation_failed)?;

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

                    let (left, right) =
                        if conflict.our_commit.timestamp < conflict.their_commit.timestamp {
                            (&file.our, &file.their)
                        } else {
                            (&file.their, &file.our)
                        };

                    Ok(SyncConflict::<'a> {
                        client: self.clone(),
                        id,
                        previous: match &file.previous {
                            Some(content) => Some(DocumentRef(
                                self.connection.document_parser.deserialize(content)?,
                            )),
                            None => None,
                        },
                        left: DocumentRef(self.connection.document_parser.deserialize(left)?),
                        right: DocumentRef(self.connection.document_parser.deserialize(right)?),
                    })
                })
                .collect::<eyre::Result<Vec<_>>>()?;

            callback(&self.query_runner, &mut documents)?;

            conflict.stage_all()?;
            operation.commit_changes()?;
        }
        Ok(())
    }

    pub fn sync<F>(&self, callback: F) -> StoreResult<()>
    where
        F: FnMut(&QueryRunner<'_>, &mut Vec<SyncConflict>) -> eyre::Result<()>,
    {
        let repository = self
            .connection
            .repository
            .lock()
            .map_err(StoreError::system_error)?;
        git_actions::fetch(&repository).map_err(StoreError::git_error)?;
        let mut rebase =
            git_actions::rebase_current_branch(&repository).map_err(StoreError::git_error)?;

        match self.rebase_action(&mut rebase, callback) {
            Ok(()) => Ok(rebase.finalize().map_err(StoreError::operation_failed)?),
            Err(e) => {
                rebase.abort().map_err(StoreError::operation_failed)?;
                Err(e)
            }
        }
        .map_err(StoreError::operation_failed)?;

        git_actions::push(&repository).map_err(StoreError::git_error)?;
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
    client: SyncRunner<'a>,
    pub id: String,
    pub previous: Option<DocumentRef>,
    pub left: DocumentRef,
    pub right: DocumentRef,
}

impl<'a> SyncConflict<'a> {
    pub fn save(&mut self, document: impl Document) -> eyre::Result<()> {
        self.client.query_runner.save(&document)?;
        Ok(())
    }
}
