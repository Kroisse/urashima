use std::fmt;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub(crate) fn unimplemented() -> Error {
        ErrorKind::Unimplemented.into()
    }
}

#[derive(Clone, Debug, Fail, PartialEq)]
pub(crate) enum ErrorKind {
    #[fail(display = "unimplemented")]
    Unimplemented,

    #[fail(display = "name error")]
    Name,

    #[fail(display = "type error")]
    Type,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

pub type Fallible<T> = Result<T, Error>;
