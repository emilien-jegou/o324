pub type Result<T> = core::result::Result<T, TaskServiceError>;

pub enum TaskServiceError {
    /// Occurs when a prefixed id filtering multiple results
    RefError(Vec<String>),
    Default(eyre::Error),
}

impl From<eyre::Error> for TaskServiceError {
    fn from(value: eyre::Error) -> Self {
        Self::Default(value)
    }
}
