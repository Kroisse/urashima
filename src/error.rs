use std::fmt;
use std::path::PathBuf;

use failure::{Backtrace, Context, Fail};

use crate::program::PackagePath;

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
    pub(crate) fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }

    pub(crate) fn unimplemented() -> Error {
        ErrorKind::Unimplemented.into()
    }

    pub(crate) fn name() -> Error {
        ErrorKind::Name.into()
    }

    pub(crate) fn loop_break() -> Error {
        ErrorKind::Break.into()
    }

    pub(crate) fn loop_continue() -> Error {
        ErrorKind::Continue.into()
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

    #[fail(display = "value error")]
    Value,

    #[fail(display = "import error")]
    Import(PackagePath),

    #[fail(display = "load error")]
    Load(PathBuf),

    #[fail(display = "unexpected break statement")]
    Break,

    #[fail(display = "unexpected continue statement")]
    Continue,
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
