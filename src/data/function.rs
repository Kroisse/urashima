use super::{Symbol, Variant};
use crate::{
    capsule::Capsule,
    error::Fallible,
    eval::{eval_in_context, Evaluate},
    // environment::Environment,
};
use urashima_ast::expr::{BlockExpression, ExprIndex};

#[derive(Clone)]
pub struct Function {
    parameters: Vec<Symbol>,
    body: BlockExpression,
    // environment: Environment,
}

impl Function {
    pub fn new(_ctx: &mut Capsule<'_>, parameters: Vec<Symbol>, body: BlockExpression) -> Self {
        Function { parameters, body }
    }

    pub fn call(&self, ctx: &mut Capsule<'_>, arguments: &[ExprIndex]) -> Fallible<Variant> {
        let Function { parameters, body } = self;
        let args: Vec<_> = arguments
            .iter()
            .map(|arg| arg.eval(ctx))
            .collect::<Result<_, _>>()?;
        let mut g = ctx.push();
        for (name, val) in parameters.iter().zip(args) {
            g.bind(&name, val);
        }
        Ok(eval_in_context(body, &mut g)?)
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::*;

    #[test]
    fn function_size() {
        assert!(mem::size_of::<Function>() <= 64);
    }
}
