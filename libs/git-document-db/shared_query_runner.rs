use crate::{Client, Document, IQueryRunner, QueryRunner, StoreResult, Transaction};
use lazy_regex::Regex;

#[derive(strum_macros::Display)]
pub enum SharedQueryRunner<'a> {
    #[strum(to_string = "Transaction")]
    Transaction(&'a Transaction<'a>),
    #[strum(to_string = "Client")]
    Client(&'a Client<'a>),
    #[strum(to_string = "QueryRunner")]
    QueryRunner(&'a QueryRunner<'a>),
}

impl<'a> SharedQueryRunner<'a> {
    pub fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>> {
        tracing::trace!("Processing 'get' command on {self} runner");
        match self {
            Self::Transaction(t) => t.get(document_id),
            Self::Client(c) => c.get(document_id),
            Self::QueryRunner(q) => q.get(document_id),
        }
    }

    pub fn get_document_list(&self) -> StoreResult<Vec<String>> {
        tracing::trace!("Processing 'get_document_list' command on {self} runner");
        match self {
            Self::Transaction(t) => t.get_document_list(),
            Self::Client(c) => c.get_document_list(),
            Self::QueryRunner(q) => q.get_document_list(),
        }
    }

    pub fn find_matching<T: Document>(&self, document_id_regex: &Regex) -> StoreResult<Vec<T>> {
        tracing::trace!("Processing 'find_matching' command on {self} runner");
        match self {
            Self::Transaction(t) => t.find_matching(document_id_regex),
            Self::Client(c) => c.find_matching(document_id_regex),
            Self::QueryRunner(q) => q.find_matching(document_id_regex),
        }
    }

    pub fn save<T: Document>(&self, document: &T) -> StoreResult<()> {
        tracing::trace!("Processing 'save' command on {self} runner");
        match self {
            Self::Transaction(t) => t.save(document),
            Self::Client(c) => c.save(document),
            Self::QueryRunner(q) => q.save(document),
        }
    }
}
