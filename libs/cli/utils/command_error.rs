use crate::utils::exit_code::ExitCode;

pub enum Error {
    ExitWithError(ExitCode, eyre::Report),
    Exit(ExitCode),
}

impl Error {
    pub fn code(&self) -> &ExitCode {
        match self {
            Error::ExitWithError(exit_code, _) => exit_code,
            Error::Exit(exit_code) => exit_code,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<E> From<E> for Error
where
    E: Into<eyre::Report>,
{
    #[track_caller]
    fn from(error: E) -> Self {
        let r: eyre::Report = error.into();
        Self::ExitWithError(ExitCode::Error, r)
    }
}
