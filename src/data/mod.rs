pub mod function;
pub mod num;
pub mod record;
pub mod value;
pub mod variant;

pub use self::{
    function::Function,
    num::{Int, Nat},
    record::Record,
    value::Value,
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
