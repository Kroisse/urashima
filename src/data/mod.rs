pub mod convert;
pub mod function;
pub mod invoke;
pub mod num;
pub mod record;
pub mod variant;

pub use self::{
    convert::FromNaru,
    function::Function,
    invoke::{Invoke, NativeMethod},
    num::{Int, Nat},
    record::Record,
    variant::Variant,
};
pub use naru_symbol::{symbol, Symbol};

#[cfg(test)]
mod test {
    use std::mem;

    use super::Symbol;

    #[test]
    fn symbol_size() {
        assert!(mem::size_of::<Symbol>() <= 8);
    }
}
