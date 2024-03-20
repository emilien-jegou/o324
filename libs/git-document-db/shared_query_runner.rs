use crate::{Client, Document, IQueryRunner, QueryRunner, StoreResult, Transaction};
use lazy_regex::Regex;

pub enum SharedQueryRunner<'a> {
    Transaction(&'a Transaction<'a>),
    Client(&'a Client<'a>),
    QueryRunner(&'a QueryRunner<'a>),
}

impl<'a> SharedQueryRunner<'a> {
    pub fn get<T: Document>(&self, document_id: &str) -> StoreResult<Option<T>> {
        match self {
            Self::Transaction(t) => t.get(document_id),
            Self::Client(c) => c.get(document_id),
            Self::QueryRunner(q) => q.get(document_id),
        }
    }

    pub fn get_document_list(&self) -> StoreResult<Vec<String>> {
        match self {
            Self::Transaction(t) => t.get_document_list(),
            Self::Client(c) => c.get_document_list(),
            Self::QueryRunner(q) => q.get_document_list(),
        }
    }

    pub fn find_matching<T: Document>(&self, document_id_regex: &Regex) -> StoreResult<Vec<T>> {
        match self {
            Self::Transaction(t) => t.find_matching(document_id_regex),
            Self::Client(c) => c.find_matching(document_id_regex),
            Self::QueryRunner(q) => q.find_matching(document_id_regex),
        }
    }

    pub fn save<T: Document>(&self, document: &T) -> StoreResult<()> {
        match self {
            Self::Transaction(t) => t.save(document),
            Self::Client(c) => c.save(document),
            Self::QueryRunner(q) => q.save(document),
        }
    }
}
