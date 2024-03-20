use crate::utils::files;
use crate::{Connection, Document, SharedQueryRunner, StoreError, StoreResult};
use lazy_regex::Regex;
use tracing::{instrument, trace};

pub trait IQueryRunner<'a>: Send + Sync {
    fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>>;
    fn get_document_list(&self) -> StoreResult<Vec<String>>;
    fn find_matching<T: Document>(
        &self,
        document_id_regex: &lazy_regex::Regex,
    ) -> StoreResult<Vec<T>>;
    fn save<T: Document>(&self, document: &T) -> StoreResult<()>;
    fn to_shared_runner(&'a self) -> SharedQueryRunner<'a>;
}

#[derive(Clone)]
pub struct QueryRunner<'a> {
    pub(crate) connection: &'a Connection,
}

impl<'a> QueryRunner<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
}

impl<'a> IQueryRunner<'a> for QueryRunner<'a> {
    #[instrument(skip(self))]
    fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>> {
        trace!("Get document");
        let mut path = self.connection.repository_path.join(document_id);
        files::add_file_extension(&mut path, self.connection.document_parser.file_extension());

        if path.exists() {
            let contents = std::fs::read_to_string(path).map_err(StoreError::corrupted_document)?;
            let data = self
                .connection
                .document_parser
                .deserialize(&contents)
                .map_err(StoreError::corrupted_document)?;
            let mut document: T =
                serde_json::from_value(data).map_err(StoreError::corrupted_document)?;
            document.set_document_id(document_id);
            Ok(Some(document))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    fn get_document_list(&self) -> StoreResult<Vec<String>> {
        trace!("Get document list");
        let extension = self.connection.document_parser.file_extension();

        // Get list of documents
        let matching_files = files::find_matching_files(
            &self.connection.repository_path,
            &Regex::new(&format!(".*{extension}$")).map_err(StoreError::operation_failed)?,
        )
        .map_err(StoreError::operation_failed)?;

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

    #[instrument(skip(self))]
    fn find_matching<T: Document>(
        &self,
        document_id_regex: &lazy_regex::Regex,
    ) -> StoreResult<Vec<T>> {
        trace!("Find matching documents");
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

    #[instrument(skip(self))]
    fn save<T: Document>(&self, document: &T) -> StoreResult<()> {
        trace!("Saving document");
        let mut path = self
            .connection
            .repository_path
            .join(document.get_document_id());

        files::add_file_extension(&mut path, self.connection.document_parser.file_extension());
        let serialized = self
            .connection
            .document_parser
            .serialize(document)
            .map_err(StoreError::corrupted_document)?;
        std::fs::write(path, serialized.as_bytes()).map_err(StoreError::operation_failed)?;
        Ok(())
    }

    fn to_shared_runner(&'a self) -> SharedQueryRunner<'a> {
        SharedQueryRunner::QueryRunner(self)
    }
}
