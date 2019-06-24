pub(crate) mod arena;
mod atomic;
mod call;
mod control_flow;
mod operator;

use std::cell::RefCell;
use std::fmt;

use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use serde_derive_urashima::DeserializeSeed;

use crate::{
    capsule::Capsule,
    data::{Symbol, Variant},
    error::{Error, Fallible},
    eval::Evaluate,
    parser::{Pairs, Parse, Rule},
};

pub use self::{
    arena::{ExprArena, ExprIndex},
    atomic::{AtomicExpression, BlockExpression},
    call::CallExpression,
    control_flow::ControlFlowExpression,
    operator::OperatorExpression,
};

pub(crate) use self::arena::Alloc;

#[derive(Clone, Debug, DeserializeSeed)]
#[serde(untagged)]
pub enum Expression {
    Atomic(AtomicExpression),
    Operator(OperatorExpression),
    Call(CallExpression),
    ControlFlow(ControlFlowExpression),
}

impl From<AtomicExpression> for Expression {
    fn from(expr: AtomicExpression) -> Self {
        Expression::Atomic(expr)
    }
}

impl From<OperatorExpression> for Expression {
    fn from(expr: OperatorExpression) -> Self {
        Expression::Operator(expr)
    }
}

impl From<CallExpression> for Expression {
    fn from(expr: CallExpression) -> Self {
        Expression::Call(expr)
    }
}

impl From<ControlFlowExpression> for Expression {
    fn from(expr: ControlFlowExpression) -> Self {
        Expression::ControlFlow(expr)
    }
}

impl Evaluate for Expression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use Expression::*;
        match self {
            Atomic(expr) => expr.eval(ctx),
            Operator(expr) => expr.eval(ctx),
            Call(expr) => expr.eval(ctx),
            ControlFlow(expr) => expr.eval(ctx),
        }
    }
}

pub(crate) struct Display<'a, T> {
    arena: &'a ExprArena,
    value: T,
}

impl<'a, T> Display<'a, T> {
    fn wrap<U>(&self, other: U) -> Display<'a, U> {
        Display {
            arena: self.arena,
            value: other,
        }
    }
}

impl<'a> fmt::Display for Display<'a, ExprIndex> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let expr = &self.arena[self.value];
        fmt::Display::fmt(&self.wrap(expr), f)
    }
}

impl<'a> fmt::Display for Display<'a, &ExprIndex> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.wrap(*self.value), f)
    }
}

impl<'a> fmt::Display for Display<'a, &Expression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Expression::Atomic(expr) => fmt::Display::fmt(&self.wrap(expr), f),
            Expression::Call(expr) => fmt::Display::fmt(&self.wrap(expr), f),
            Expression::Operator(expr) => fmt::Display::fmt(&self.wrap(expr), f),
            Expression::ControlFlow(expr) => unimplemented!(),
        }
    }
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        Operator::new(Rule::op_1, Assoc::Left),
        Operator::new(Rule::op_2, Assoc::Left),
        Operator::new(Rule::punctuation, Assoc::Left),
    ]);
}

impl Parse for Expression {
    const RULE: Rule = Rule::expression;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let cell = RefCell::new(arena);
        PREC_CLIMBER.climb(
            pairs,
            |p| match p.as_rule() {
                Rule::operand_expression => {
                    parse_operand_expression(&mut *cell.borrow_mut(), p.into_inner())
                }
                _ => unreachable!("{:?}", p),
            },
            |left, op, right| {
                let mut arena = cell.borrow_mut();
                Ok(OperatorExpression::Infix(
                    op.as_str().into(),
                    arena.insert(left?),
                    arena.insert(right?),
                )
                .into())
            },
        )
    }
}

fn parse_operand_expression(arena: &mut ExprArena, mut pairs: Pairs<'_>) -> Fallible<Expression> {
    let mut expr: Expression = if let Some(head) = pairs.next() {
        match head.as_rule() {
            Rule::boolean => match head.as_str() {
                "false" => AtomicExpression::False.into(),
                "true" => AtomicExpression::True.into(),
                _ => unreachable!(),
            },
            Rule::numeric => {
                let num = head.as_str().parse()?;
                AtomicExpression::Integral(num).into()
            }
            Rule::fn_expression => {
                let (parameters, body) = parse_fn_expression(&mut *arena, head.into_inner())?;
                AtomicExpression::Fn { parameters, body }.into()
            }
            Rule::name => AtomicExpression::Name(head.as_str().into()).into(),
            _ => {
                return Err(Error::unimplemented());
            }
        }
    } else {
        unreachable!();
    };
    for rest in pairs {
        match rest.as_rule() {
            Rule::method_call => {
                let (method, args) = parse_method_call(&mut *arena, rest.into_inner())?;
                expr = CallExpression::invoke(arena.insert(expr), method, args).into();
            }
            _ => unreachable!("{:?}", rest),
        }
    }
    Ok(expr)
}

fn parse_fn_expression(
    arena: &mut ExprArena,
    pairs: Pairs<'_>,
) -> Fallible<(Vec<Symbol>, BlockExpression)> {
    let mut params = vec![];
    let mut block: Option<BlockExpression> = None;
    for item in pairs {
        match item.as_rule() {
            Rule::fn_param => {
                params.push(item.as_str().into());
            }
            Rule::grouping_brace => {
                if block.is_none() {
                    block = Some(BlockExpression::from_pairs(&mut *arena, item.into_inner())?);
                } else {
                    unreachable!("{:?}", item);
                }
            }
            _ => unreachable!("{:?}", item),
        }
    }
    Ok((params, block.expect("unreachable")))
}

fn parse_method_call(
    arena: &mut ExprArena,
    mut pairs: Pairs<'_>,
) -> Fallible<(Symbol, Vec<ExprIndex>)> {
    let method_name: Symbol = if let Some(head) = pairs.next() {
        match head.as_rule() {
            Rule::name => head.as_str().into(),
            _ => unreachable!(),
        }
    } else {
        unreachable!();
    };
    let arguments = pairs
        .map(|rest| match rest.as_rule() {
            Rule::expression => ExprIndex::from_pair(&mut *arena, rest),
            _ => unreachable!("{:?}", rest),
        })
        .collect::<Fallible<_>>()?;
    Ok((method_name, arguments))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expr::ExprArena;

    #[test]
    fn atomic_true() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "true").unwrap(),
            Expression::Atomic(AtomicExpression::True) => {}
        );
    }

    #[test]
    fn atomic_false() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "false").unwrap(),
            Expression::Atomic(AtomicExpression::False) => {}
        );
    }

    #[test]
    fn atomic_fn_simple_1() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "fn {}").unwrap(),
            Expression::Atomic(AtomicExpression::Fn { parameters, body }) => {
                assert_eq!(parameters.len(), 0);
                assert_eq!(body.statements().len(), 0);
            }
        );
    }

    #[test]
    fn atomic_fn_simple_2() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "fn (a) { a + 1 }").unwrap(),
            Expression::Atomic(AtomicExpression::Fn { parameters, body }) => {
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters, vec![Symbol::from("a")]);
                assert_eq!(body.statements().len(), 0);
            }
        );
    }
}
