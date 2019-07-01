use crate::{
    arena::Index,
    data::Variant,
    error::Fallible,
};


pub trait Type {
    fn invoke(arguments: &[Index<Variant>]) -> Fallible<()>;
}
