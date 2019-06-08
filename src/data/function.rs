use super::{Symbol, Variant};
use crate::{
    capsule::Capsule,
    environment::Environment,
    error::Fallible,
    eval::Evaluate,
    expr::{BlockExpression, ExprIndex},
};

#[derive(Clone, Debug)]
pub struct Function {
    parameters: Vec<Symbol>,
    closure: Environment,
    body: BlockExpression,
}

impl Function {
    pub fn new(_ctx: &mut Capsule, parameters: Vec<Symbol>, body: BlockExpression) -> Self {
        let closure = Environment::default();
        Function {
            parameters,
            closure,
            body,
        }
    }

    pub fn call(&self, ctx: &mut Capsule, arguments: &[ExprIndex]) -> Fallible<Variant> {
        let Function {
            parameters,
            body,
            closure,
        } = self;
        let args: Vec<_> = arguments
            .iter()
            .map(|arg| arg.eval(ctx))
            .collect::<Result<_, _>>()?;
        let mut g = ctx.push();
        g.load(&closure);
        for (name, val) in parameters.iter().zip(args) {
            g.bind(&name, val);
        }
        Ok(body.eval_in_context(&mut g)?)
    }
}
