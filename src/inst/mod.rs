mod translate;

use crate::data::{Int, Nat, Symbol};

/// Instruction code

pub type LocalIndex = u32;

#[derive(PartialEq)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
pub enum Instruction {
    Unreachable,
    Nop,
    Block,
    Loop(Option<u32>),
    If,
    Else,
    End,
    Break(Option<u32>),
    BreakIf(Option<u32>),
    Return,
    Call(u32),
    Invoke(u8, u32),

    Discard,
    // Select,
    LocalGet(LocalIndex),
    LocalSet(LocalIndex),
    LocalTee(LocalIndex),

    BoolConst(bool),
    I32Const(i32),
    N32Const(u32),
    IntConst(Int),
    NatConst(Nat),
    StrConst(String),

    MethodRef(Symbol),
}
