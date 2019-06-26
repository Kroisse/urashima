mod arena;
mod block;
mod call;
mod control_flow;
mod function;

use std::cell::RefCell;

use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use crate::{
    error::{Error, Fallible},
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
};

pub use self::{
    arena::{ExprArena, ExprIndex},
    block::BlockExpression,
    call::{CallExpression, InvokeExpression},
    control_flow::{IfExpression, LoopExpression},
    function::{FunctionExpression, Parameter},
};

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub enum Expression {
    // Atomic
    False,
    True,
    Integral(i64),
    Str(String),
    Name(Symbol),

    Record(#[cfg_attr(feature = "deserialize", serde(state))] Vec<(Symbol, ExprIndex)>),
    Block(#[cfg_attr(feature = "deserialize", serde(state))] BlockExpression),
    Fn(#[cfg_attr(feature = "deserialize", serde(state))] FunctionExpression),

    // Operator
    New(#[cfg_attr(feature = "deserialize", serde(state))] ExprIndex),
    Infix(
        Symbol,
        #[cfg_attr(feature = "deserialize", serde(state))] ExprIndex,
        #[cfg_attr(feature = "deserialize", serde(state))] ExprIndex,
    ),
    Call(#[cfg_attr(feature = "deserialize", serde(state))] CallExpression),
    Invoke(#[cfg_attr(feature = "deserialize", serde(state))] InvokeExpression),

    // Control flow
    If(#[cfg_attr(feature = "deserialize", serde(state))] IfExpression),
    Loop(#[cfg_attr(feature = "deserialize", serde(state))] LoopExpression),
}

impl Expression {
    pub(crate) fn unit() -> Self {
        Expression::Record(vec![])
    }

    pub(crate) fn call(callee: ExprIndex, arguments: Vec<ExprIndex>) -> Self {
        Expression::Call(CallExpression::new(callee, arguments))
    }

    pub(crate) fn invoke(receiver: ExprIndex, method: Symbol, arguments: Vec<ExprIndex>) -> Self {
        Expression::Invoke(InvokeExpression::new(receiver, method, arguments))
    }

    pub(crate) fn is_unit(&self) -> bool {
        if let Expression::Record(fields) = self {
            fields.is_empty()
        } else {
            false
        }
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

impl<'a> Print for Expression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        use Expression::*;
        match self {
            False => f.write_str("false"),
            True => f.write_str("true"),
            Integral(i) => write!(f, "{}", i),
            Str(s) => write!(f, "{}", s),
            Name(name) => f.write_str(&name),
            Block(expr) => Print::fmt(expr, f),
            Fn(expr) => Print::fmt(expr, f),

            Infix(op, a, b) => write!(f, "{} {} {}", f.display(a), op, f.display(b)),
            New(expr) => write!(f, "new {}", f.display(expr)),
            Call(expr) => Print::fmt(expr, f),
            Invoke(expr) => Print::fmt(expr, f),

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
                _ => unreachable!(),
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
            Rule::grouping_brace => {
                let expr = BlockExpression::from_pairs(&mut *arena, head.into_inner())?;
                Expression::Block(expr)
            }
            Rule::fn_expression => {
                let expr = FunctionExpression::from_pairs(&mut *arena, head.into_inner())?;
                Expression::Fn(expr)
            }
            Rule::name => Expression::Name(head.as_str().into()),

            Rule::if_expression => {
                let expr = IfExpression::from_pairs(&mut *arena, head.into_inner())?;
                Expression::If(expr)
            }
            Rule::loop_expression => {
                let expr = LoopExpression::from_pairs(&mut *arena, head.into_inner())?;
                Expression::Loop(expr)
            }
            _ => {
                return Err(Error::unimplemented());
            }
        }
    } else {
        unreachable!();
    };
    for rest in pairs {
        match rest.as_rule() {
            Rule::call_arguments => {
                let args = parse_call_arguments(arena, rest.into_inner())?;
                expr = Expression::call(arena.insert(expr), args);
            }
            Rule::method_call => {
                let (method, args) = parse_method_call(&mut *arena, rest.into_inner())?;
                expr = Expression::invoke(arena.insert(expr), method, args);
            }
            _ => unreachable!(),
        }
    }
    Ok(expr)
}

fn parse_call_arguments(
    arena: &mut ExprArena,
    pairs: Pairs<'_>,
) -> Fallible<Vec<ExprIndex>> {
    pairs
        .map(|rest| match rest.as_rule() {
            Rule::expression => ExprIndex::from_pair(&mut *arena, rest),
            _ => unreachable!(),
        })
        .collect::<Fallible<_>>()
}

fn parse_method_call(
    arena: &mut ExprArena,
    mut pairs: Pairs<'_>,
) -> Fallible<(Symbol, Vec<ExprIndex>)> {
    if let (Some(name), Some(args), None) = (pairs.next(), pairs.next(), pairs.next()) {
        if name.as_rule() != Rule::name || args.as_rule() != Rule::call_arguments {
            unreachable!();
        }
        let method_name = name.as_str().into();
        let arguments = parse_call_arguments(arena, pairs)?;
        Ok((method_name, arguments))
    } else {
        unreachable!()
    }
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
                assert_eq!(parameters, vec![Parameter::from("a")]);
                assert_eq!(body.statements().len(), 0);
            }
        );
    }
}
