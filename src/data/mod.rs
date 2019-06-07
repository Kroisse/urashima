pub mod function;
pub mod record;
pub mod variant;

#[macro_use]
mod internal {
    include!(concat!(env!("OUT_DIR"), "/symbol.rs"));
}

pub use self::{function::Function, internal::Symbol, record::Record, variant::Variant};
