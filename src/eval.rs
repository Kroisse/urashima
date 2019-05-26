use crate::capsule::Context;
use crate::error::Fallible;

pub trait Evaluate {
    type Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value>;
}
