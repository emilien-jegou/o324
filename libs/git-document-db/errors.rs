use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("couldn't parse document {0}")]
    IoError(std::io::Error),
    #[error("couldn't parse store document: {0}")]
    CorruptedDocument(String),
    #[error("couldn't parse document {0}")]
    DocumentParseError(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("git error: {0}")]
    GitError(String),
    #[error("unknown store error")]
    Unknown,
}


pub type StoreResult<T> = Result<T, StoreError>;
