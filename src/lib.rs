//  Copyright 2019 Eunchong Yu <kroisse@gmail.com>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![deny(rust_2018_idioms)]
#![cfg_attr(test, recursion_limit = "128")]

#[macro_use]
mod data;

mod environment;
mod eval;
mod inst;

pub mod capsule;
pub mod error;
pub mod runtime;

pub use crate::capsule::Capsule;
pub use crate::error::{Error, Fallible};
pub use crate::runtime::Runtime;
