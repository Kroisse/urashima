pub mod record;

#[macro_use]
mod internal {
    include!(concat!(env!("OUT_DIR"), "/symbol.rs"));
}

pub use self::internal::Symbol;
