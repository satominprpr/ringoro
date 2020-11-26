use std::sync::Arc;

use err_derive::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Error)]
#[error(display = "{}", kind)]
#[non_exhaustive]
pub struct Error {
    pub kind: Arc<ErrorKind>,
}

impl Error {
    pub fn new(e: Arc<ErrorKind>) -> Error {
        Error { kind: e }
    }
}

impl std::ops::Deref for Error {
    type Target = Arc<ErrorKind>;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl<E> From<E> for Error
where
    ErrorKind: From<E>,
{
    fn from(err: E) -> Self {
        Self {
            kind: Arc::new(err.into()),
        }
    }
}

type IoError = std::io::Error;
type IoErrorKind = std::io::ErrorKind;

impl From<Error> for IoError {
    fn from(err: Error) -> IoError {
        match Arc::try_unwrap(err.kind) {
            Ok(ErrorKind::Io(ioerr)) => ioerr,
            Ok(err) => IoError::new::<Error>(IoErrorKind::Other, err.into()),
            Err(e) => std::io::Error::new(IoErrorKind::Other, Box::new(Error::new(e))),
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error(display = "{}", _0)]
    Mongodb(#[error(source)] mongodb::error::Error),

    #[error(display = "{}", _0)]
    Io(#[error(source)] std::io::Error),

    #[error(display = "{}", _0)]
    ConfigFromEnv(#[error(source)] envy::Error),

    #[error(display = "{}", _0)]
    Env(#[error(source)] std::env::VarError),
}
