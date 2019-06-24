use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;

use failure::{Backtrace, Context, Fail};

use crate::{
    data::{symbol, Symbol},
    parser::Rule,
    program::PackagePath,
};

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

    pub(crate) fn runtime() -> Error {
        ErrorKind::Runtime.into()
    }

    pub(crate) fn unimplemented() -> Error {
        ErrorKind::Unimplemented.into()
    }

    pub(crate) fn name() -> Error {
        ErrorKind::Name.into()
    }

    pub(crate) fn invalid_type(expected: impl Into<Symbol>) -> Error {
        ErrorKind::Type {
            expected: expected.into(),
        }
        .into()
    }

    pub(crate) fn value(reason: impl Into<Cow<'static, str>>) -> Error {
        ErrorKind::Value {
            reason: reason.into(),
        }
        .into()
    }

    pub(crate) fn import(path: &PackagePath) -> Error {
        ErrorKind::Import(path.clone()).into()
    }

    pub(crate) fn load(path: impl Into<PathBuf>) -> Error {
        ErrorKind::Load(path.into()).into()
    }

    pub(crate) fn loop_break() -> Error {
        ErrorKind::ControlFlow(ControlFlow::Break).into()
    }

    pub(crate) fn loop_continue() -> Error {
        ErrorKind::ControlFlow(ControlFlow::Continue).into()
    }

    pub(crate) fn is_unexpected(&self) -> bool {
        if let ErrorKind::UnexpectedRule { .. } = self.inner.get_context() {
            true
        } else {
            false
        }
    }

    pub(crate) fn as_control_flow(&self) -> Option<&ControlFlow> {
        if let ErrorKind::ControlFlow(cf) = self.inner.get_context() {
            Some(cf)
        } else {
            None
        }
    }
}

#[derive(Debug, Fail)]
enum ErrorKind {
    #[fail(display = "parse error: {}", _0)]
    Parse(String),

    #[fail(display = "expected rule = {:?}, found = {:?}", expected, found)]
    UnexpectedRule { expected: Rule, found: Rule },

    #[fail(display = "runtime error")]
    Runtime,

    #[fail(display = "unimplemented")]
    Unimplemented,

    #[fail(display = "name error")]
    Name,

    #[fail(display = "type error: expected '{}'", expected)]
    Type { expected: Symbol },

    #[fail(display = "value error: {}", reason)]
    Value { reason: Cow<'static, str> },

    #[fail(display = "import error")]
    Import(PackagePath),

    #[fail(display = "load error")]
    Load(PathBuf),

    #[fail(display = "unexpected {} statement", _0)]
    ControlFlow(ControlFlow),
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

#[derive(Debug)]
pub enum ControlFlow {
    Break,
    Continue,
}

impl ControlFlow {
    pub fn as_symbol(&self) -> Symbol {
        match self {
            ControlFlow::Break => symbol!("break"),
            ControlFlow::Continue => symbol!("continue"),
        }
    }
}

impl fmt::Display for ControlFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_symbol())
    }
}
