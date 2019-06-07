use super::{Symbol, Variant};
use crate::{
    capsule::Capsule,
    environment::Environment,
    error::Fallible,
    eval::Evaluate,
    expr::{BlockExpression, Expression},
};

#[derive(Clone, Debug)]
pub struct Function {
    parameters: Vec<Symbol>,
    closure: Environment,
    body: BlockExpression,
}

impl Function {
    pub fn new(ctx: &mut Capsule, parameters: Vec<Symbol>, body: BlockExpression) -> Self {
        let closure = Environment::default();
        Function {
            parameters,
            closure,
            body,
        }
    }

    pub fn call(&self, ctx: &mut Capsule, arguments: &[Expression]) -> Fallible<Variant> {
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
            g.bind(name.clone(), val);
        }
        Ok(body.eval_in_context(&mut g)?)
    }
}
