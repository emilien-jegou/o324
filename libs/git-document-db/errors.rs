use lazy_regex::regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("couldn't parse store document: {0}")]
    CorruptedDocument(ErrorData),
    #[error("operation failed: {0}")]
    OperationFailed(ErrorData),
    #[error("lock failed: {0}")]
    LockError(ErrorData),
    #[error("git error: {0}")]
    GitError(ErrorData),
    #[error("unknown system error {0}")]
    SystemError(ErrorData),
}

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub struct ErrorData(String);

impl std::fmt::Display for ErrorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_errordata {
    ($type:ty) => {
        impl From<$type> for ErrorData {
            fn from(error: $type) -> Self {
                Self(format!("{}", error))
            }
        }
    };
}

impl_errordata!(eyre::Report);
impl_errordata!(serde_json::Error);
impl_errordata!(regex::Error);
impl_errordata!(std::io::Error);
impl_errordata!(git2::Error);
impl_errordata!(&str);

impl<T> From<std::sync::PoisonError<T>> for ErrorData {
    fn from(error: std::sync::PoisonError<T>) -> Self {
        Self(format!("{}", error))
    }
}

impl StoreError {
    pub fn operation_failed<T: Into<ErrorData>>(error: T) -> Self {
        Self::OperationFailed(error.into())
    }

    pub fn lock_error<T: Into<ErrorData>>(error: T) -> Self {
        Self::LockError(error.into())
    }

    pub fn git_error<T: Into<ErrorData>>(error: T) -> Self {
        Self::GitError(error.into())
    }

    pub fn corrupted_document<T: Into<ErrorData>>(error: T) -> Self {
        Self::CorruptedDocument(error.into())
    }

    pub fn system_error<T: Into<ErrorData>>(error: T) -> Self {
        Self::SystemError(error.into())
    }
}
