use urashima_ast::{
    expr::{
        CallExpression, ExprArena, ExprIndex, Expression, IfExpression, InvokeExpression,
        LoopExpression,
    },
    statement::Statement,
};

use super::Instruction;
use crate::data::Int;

struct Ctx<'a> {
    inst: Vec<Instruction>,
    arena: &'a ExprArena,
}

trait Translate {
    fn translate(&self, ctx: &mut Ctx<'_>);
}

impl Translate for ExprIndex {
    fn translate(&self, ctx: &mut Ctx<'_>) {
        ctx.arena[*self].translate(ctx)
    }
}

impl Translate for Expression {
    fn translate(&self, ctx: &mut Ctx<'_>) {
        use Expression::*;
        match self {
            False => {
                ctx.inst.push(Instruction::BoolConst(false));
            }
            True => {
                ctx.inst.push(Instruction::BoolConst(true));
            }
            Integral(val) => {
                ctx.inst.push(Instruction::IntConst(Int::from(*val)));
            }
            Str(val) => {
                ctx.inst.push(Instruction::StrConst(val.clone()));
            }
            Infix(op, left, right) => {
                left.translate(ctx);
                right.translate(ctx);
                ctx.inst.push(Instruction::MethodRef(op.clone()));
                ctx.inst.push(Instruction::Invoke(2, 0));
            }
            Call(CallExpression {
                callee, arguments, ..
            }) => {
                callee.translate(ctx);
                for a in arguments {
                    a.translate(ctx);
                }
                ctx.inst.push(Instruction::Call(arguments.len() as u32));
            }
            Invoke(InvokeExpression {
                receiver,
                method,
                arguments,
                ..
            }) => {
                receiver.translate(ctx);
                for a in arguments {
                    a.translate(ctx);
                }
                ctx.inst.push(Instruction::MethodRef(method.clone()));
                ctx.inst
                    .push(Instruction::Invoke(1, arguments.len() as u32));
            }
            If(IfExpression {
                cond,
                then_blk,
                else_blk,
                ..
            }) => {
                cond.translate(ctx);
                ctx.inst.push(Instruction::If);
                for s in then_blk {
                    s.translate(ctx);
                }
                if let Some(else_blk) = else_blk {
                    ctx.inst.push(Instruction::Else);
                    for s in else_blk {
                        s.translate(ctx);
                    }
                }
                ctx.inst.push(Instruction::End);
            }
            Loop(LoopExpression { blk, .. }) => {
                ctx.inst.push(Instruction::Loop(None));
                for s in blk {
                    s.translate(ctx);
                }
                ctx.inst.push(Instruction::End);
            }
            _ => unimplemented!(),
        }
    }
}

impl Translate for Statement {
    fn translate(&self, ctx: &mut Ctx<'_>) {
        use Statement::*;
        match self {
            Expr(expr) => {
                expr.translate(ctx);
            }
            Break => {
                ctx.inst.push(Instruction::Break(None));
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use urashima_ast::{expr::ExprArena, parse};

    use super::*;
    use crate::data::Symbol;

    macro_rules! assert_translate {
        ($name:ident: $t:ty = $code:literal; $($inst:expr,)*) => {
            #[test]
            fn $name() {
                use Instruction::*;
                let s = $code;
                let mut arena = ExprArena::new();
                let expr: $t = parse(&mut arena, s).unwrap();
                let mut ctx = Ctx {
                    inst: vec![],
                    arena: &arena,
                };
                expr.translate(&mut ctx);
                assert_eq!(
                    &ctx.inst,
                    &[$(
                        $inst,
                    )*],
                );
            }
        }
    }

    assert_translate! {
        infix_simple: Expression = "1 + 2";
        IntConst(Int::from(1)),
        IntConst(Int::from(2)),
        MethodRef(Symbol::from("+")),
        Invoke(2, 0),
    }

    assert_translate! {
        if_simple: Expression = "if true { 42 }";
        BoolConst(true),
        If,
        IntConst(Int::from(42)),
        End,
    }

    assert_translate! {
        if_simple_2: Expression = "if 1 < 2 { 42 }";
        IntConst(Int::from(1)),
        IntConst(Int::from(2)),
        MethodRef(Symbol::from("<")),
        Invoke(2, 0),
        If,
        IntConst(Int::from(42)),
        End,
    }

    assert_translate! {
        if_else_simple: Expression = "if true { 42 } else { 43 }";
        BoolConst(true),
        If,
        IntConst(Int::from(42)),
        Else,
        IntConst(Int::from(43)),
        End,
    }
}
