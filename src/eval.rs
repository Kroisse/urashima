use crate::capsule::Capsule;
use crate::error::Fallible;

pub trait Evaluate {
    type Value;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value>;
}
