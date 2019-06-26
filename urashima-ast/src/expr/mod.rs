pub(crate) mod arena;
mod block;
mod call;
mod control_flow;
mod function;

use std::cell::RefCell;
use std::fmt;

use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive_urashima::DeserializeSeed;

use crate::{
    error::{Error, Fallible},
    parser::{Pairs, Parse, Rule},
};

pub use self::{
    arena::{Alloc, ExprArena, ExprIndex},
    block::BlockExpression,
    call::{CallExpression, InvokeExpression},
    control_flow::{IfExpression, LoopExpression},
    function::FunctionExpression,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub enum Expression {
    // Atomic
    False,
    True,
    Integral(i64),
    Str(String),
    Name(Symbol),
    Record(Vec<(Symbol, ExprIndex)>),
    Block(BlockExpression),
    Fn(FunctionExpression),

    // Operator
    New(ExprIndex),
    Infix(Symbol, ExprIndex, ExprIndex),
    Call(CallExpression),
    Invoke(InvokeExpression),

    // Control flow
    If(IfExpression),
    Loop(LoopExpression),
}

impl Expression {
    pub(crate) fn unit() -> Self {
        Expression::Record(vec![])
    }

    pub(crate) fn invoke(receiver: ExprIndex, method: Symbol, arguments: Vec<ExprIndex>) -> Self {
        Expression::Invoke(InvokeExpression::new(receiver, method, arguments))
    }
}

impl From<BlockExpression> for Expression {
    fn from(expr: BlockExpression) -> Self {
        Expression::Block(expr)
    }
}

impl From<CallExpression> for Expression {
    fn from(expr: CallExpression) -> Self {
        Expression::Call(expr)
    }
}

impl From<InvokeExpression> for Expression {
    fn from(expr: InvokeExpression) -> Self {
        Expression::Invoke(expr)
    }
}

impl From<IfExpression> for Expression {
    fn from(expr: IfExpression) -> Self {
        Expression::If(expr)
    }
}

impl From<LoopExpression> for Expression {
    fn from(expr: LoopExpression) -> Self {
        Expression::Loop(expr)
    }
}

pub(crate) struct Display<'a, T> {
    arena: &'a ExprArena,
    value: T,
}

impl<'a, T> Display<'a, T> {
    pub(crate) fn new(arena: &'a ExprArena, value: T) -> Self {
        Display { arena, value }
    }

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
            Expression::False => fmt::Display::fmt("false", f),
            Expression::True => fmt::Display::fmt("true", f),
            Expression::Integral(i) => fmt::Display::fmt(&i, f),
            Expression::Str(s) => fmt::Display::fmt(&s, f),

            Expression::Infix(op, a, b) => write!(f, "{} {} {}", self.wrap(*a), op, self.wrap(*b)),
            Expression::New(expr) => write!(f, "new {}", self.wrap(*expr)),
            Expression::Call(expr) => fmt::Display::fmt(&self.wrap(expr), f),
            Expression::Invoke(expr) => fmt::Display::fmt(&self.wrap(expr), f),

            // Expression::ControlFlow(expr) => unimplemented!(),

            _ => unimplemented!(),
        }
    }
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        Operator::new(Rule::op_1, Assoc::Left),
        Operator::new(Rule::op_2, Assoc::Left),
        Operator::new(Rule::op_3, Assoc::Left),
        Operator::new(Rule::op_4, Assoc::Left),
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
                Ok(Expression::Infix(
                    op.as_str().into(),
                    arena.insert(left?),
                    arena.insert(right?),
                ))
            },
        )
    }
}

fn parse_operand_expression(arena: &mut ExprArena, mut pairs: Pairs<'_>) -> Fallible<Expression> {
    let mut expr: Expression = if let Some(head) = pairs.next() {
        match head.as_rule() {
            Rule::boolean => match head.as_str() {
                "false" => Expression::False,
                "true" => Expression::True,
                _ => unreachable!(),
            },
            Rule::numeric => {
                let num = head.as_str().parse()?;
                Expression::Integral(num)
            }
            Rule::string => {
                let text = head.as_str();
                Expression::Str(text.to_string())
            }
            Rule::fn_expression => {
                let expr = FunctionExpression::from_pairs(&mut *arena, head.into_inner())?;
                Expression::Fn(expr)
            }
            Rule::name => Expression::Name(head.as_str().into()),
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
                expr = Expression::invoke(arena.insert(expr), method, args);
            }
            _ => unreachable!("{:?}", rest),
        }
    }
    Ok(expr)
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
            Expression::True => {}
        );
    }

    #[test]
    fn atomic_false() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "false").unwrap(),
            Expression::False => {}
        );
    }

    #[test]
    fn atomic_str_simple_1() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "'Hello world!'").unwrap(),
            Expression::Str(s) => {
                assert_eq!(s, "Hello world!");
            }
        );
    }

    #[test]
    fn atomic_str_simple_2() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#""Hello, world!""#).unwrap(),
            Expression::Str(s) => {
                assert_eq!(s, "Hello, world!");
            }
        );
    }

    #[test]
    fn atomic_fn_simple_1() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, "fn {}").unwrap(),
            Expression::Fn(FunctionExpression { parameters, body, .. }) => {
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
            Expression::Fn(FunctionExpression { parameters, body, .. }) => {
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters, vec![Symbol::from("a")]);
                assert_eq!(body.statements().len(), 0);
            }
        );
    }
}
