#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![deny(rust_2018_idioms)]

mod capsule;
mod environment;
mod error;
mod eval;
mod expr;
mod statement;

pub use crate::capsule::Capsule;
pub use crate::error::{Error, Fallible};
