use std::fmt;

use failure::{Backtrace, Context, Fail};
use urashima_util::Symbol;

use crate::parser::Rule;

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
    pub(crate) fn from_de<E>(err: E) -> Self
    where
        E: serde::de::Error,
    {
        ErrorKind::Parse(err.to_string()).into()
    }

    pub(crate) fn unexpected(expected: Rule, found: Rule) -> Self {
        ErrorKind::UnexpectedRule { expected, found }.into()
    }

    pub(crate) fn unimplemented() -> Error {
        ErrorKind::Unimplemented.into()
    }

    pub(crate) fn name(name: impl Into<Symbol>) -> Error {
        ErrorKind::Name { name: name.into() }.into()
    }

    pub(crate) fn invalid_type(expected: impl Into<Symbol>) -> Error {
        ErrorKind::Type {
            expected: expected.into(),
        }
        .into()
    }

    pub(crate) fn is_unexpected(&self) -> bool {
        if let ErrorKind::UnexpectedRule { .. } = self.inner.get_context() {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Fail)]
enum ErrorKind {
    #[fail(display = "parse error: {}", _0)]
    Parse(String),

    #[fail(display = "expected rule = {:?}, found = {:?}", expected, found)]
    UnexpectedRule { expected: Rule, found: Rule },

    #[fail(display = "unimplemented")]
    Unimplemented,

    #[fail(display = "name error: {}", name)]
    Name { name: Symbol },

    #[fail(display = "type error: expected '{}'", expected)]
    Type { expected: Symbol },
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Error { inner }
    }
}

impl<R: pest::RuleType> From<pest::error::Error<R>> for Error {
    fn from(err: pest::error::Error<R>) -> Self {
        ErrorKind::Parse(err.to_string()).into()
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        ErrorKind::Parse(err.to_string()).into()
    }
}

pub type Fallible<T> = Result<T, Error>;
