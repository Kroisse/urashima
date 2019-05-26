#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![deny(rust_2018_idioms)]
#![cfg_attr(test, recursion_limit = "128")]

mod capsule;
mod environment;
mod error;
mod eval;
mod expr;
mod statement;

pub use crate::capsule::Capsule;
pub use crate::error::{Error, Fallible};
