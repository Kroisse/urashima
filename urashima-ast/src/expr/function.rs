use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive::Deserialize;
#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use super::{BlockExpression, ExprArena};
use crate::{
    error::Fallible,
    find::Find,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
    span::{Position, Span, Spanned},
};

#[derive(Clone)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct FunctionExpression {
    pub parameters: Vec<Parameter>,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub body: BlockExpression,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

#[derive(Clone, PartialEq)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct Parameter {
    pub name: Spanned<Symbol>,
}

impl Parameter {
    pub fn name(&self) -> Symbol {
        self.name.node.clone()
    }
}

impl Parse for FunctionExpression {
    const RULE: Rule = Rule::fn_expression;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let mut parameters = vec![];
        let mut block: Option<BlockExpression> = None;
        for item in pairs {
            match item.as_rule() {
                Rule::fn_param => {
                    parameters.push(Parameter {
                        name: Spanned::new(&item.as_span(), item.as_str().into()),
                    });
                }
                Rule::grouping_brace => {
                    if block.is_none() {
                        block = Some(BlockExpression::from_pairs(
                            &mut *arena,
                            item.as_span(),
                            item.into_inner(),
                        )?);
                    } else {
                        unreachable!();
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(FunctionExpression {
            parameters,
            body: block.expect("unreachable"),
            __opaque: (),
        })
    }
}

impl<'a> Print for FunctionExpression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        f.write_str("fn")?;
        if !self.parameters.is_empty() {
            write!(f, "({})", f.display_seq(&self.parameters, ", "))?;
        }
        f.write_str(" ")?;
        Print::fmt(&self.body, f)
    }
}

impl<'a> Print for Parameter {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        f.write_str(&self.name.node)
    }
}

impl Find for FunctionExpression {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(FunctionExpression)");
        self.parameters
            .binary_search_by(|param| param.name.span.cmp_pos(&pos))
            .ok()
            .and_then(|i| self.parameters[i].find_span(pos, arena))
            .or_else(|| self.body.find_span(pos, arena))
    }
}

impl Find for Parameter {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(Parameter)");
        self.name.span.find_span(pos, arena)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{parse, program::ScriptProgram};

    #[test]
    fn inverse() {
        let s = r#"f := fn {
    x println()
}
f()
"#;
        let mut arena = ExprArena::new();
        let prog: ScriptProgram = parse(&mut arena, &s).unwrap();
        let printed = prog.display(&arena).to_string();
        assert_eq!(s, printed);
    }
}
