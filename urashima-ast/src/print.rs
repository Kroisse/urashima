use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::expr::{ExprArena, ExprIndex, Expression};

pub use std::fmt::{Error, Write};

pub type Result<T = ()> = std::result::Result<T, Error>;

pub struct Formatter<'a> {
    inner: &'a mut fmt::Formatter<'a>,
    arena: &'a ExprArena,
    state: Rc<RefCell<State>>,
}

struct State {
    start: bool,
    indent: u16,
}

impl<'a> Formatter<'a> {
    fn new<'b: 'a>(
        f: &'a mut fmt::Formatter<'b>,
        arena: &'a ExprArena,
        state: Rc<RefCell<State>>,
    ) -> Self {
        let inner = unsafe {
            std::mem::transmute::<&'a mut fmt::Formatter<'b>, &'a mut fmt::Formatter<'a>>(f)
        };
        Formatter {
            inner,
            arena,
            state,
        }
    }

    fn write_indent(&mut self) -> Result {
        let mut st = self.state.borrow_mut();
        if st.start {
            st.start = false;
            for _ in dbg!(0..st.indent) {
                self.inner.write_str("    ")?;
            }
        }
        Ok(())
    }

    pub fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result {
        self.write_indent()?;
        self.inner.write_fmt(fmt)
    }

    pub fn write_str(&mut self, data: &str) -> Result {
        self.inner.write_str(data)
    }

    pub fn next_line(&mut self) -> Result {
        self.inner.write_char('\n')?;
        self.state.borrow_mut().start = true;
        Ok(())
    }

    pub fn indent(&mut self, blk: impl FnOnce(&mut Formatter<'a>) -> Result) -> Result {
        self.state.borrow_mut().indent += 1;
        let ret = blk(self);
        self.state.borrow_mut().indent -= 1;
        ret
    }

    pub fn get(&self, idx: ExprIndex) -> Result<&'a Expression> {
        self.arena.get(idx).ok_or(Error)
    }

    pub fn display<'t, T>(&self, data: &'t T) -> Display<'a, &'t T>
    where
        T: Print + ?Sized + 't,
    {
        Display {
            data,
            arena: self.arena,
            state: Rc::clone(&self.state),
        }
    }

    pub(crate) fn display_seq<'t, 's, T>(
        &self,
        data: &'t [T],
        separator: &'s str,
    ) -> Display<'a, Sequence<'t, 's, T>>
    where
        T: Print + 't,
    {
        Display {
            data: Sequence::new(data, separator),
            arena: self.arena,
            state: Rc::clone(&self.state),
        }
    }
}

pub trait Print {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result;

    fn display<'a, 'b>(&'a self, arena: &'b ExprArena) -> Display<'b, &'a Self> {
        let state = Rc::new(RefCell::new(State {
            start: true,
            indent: 0,
        }));
        Display {
            data: self,
            arena,
            state,
        }
    }
}

impl<T> Print for &T
where
    T: Print + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Print::fmt(*self, f)
    }
}

pub struct Display<'a, T> {
    data: T,
    arena: &'a ExprArena,
    state: Rc<RefCell<State>>,
}

impl<'a, T> fmt::Display for Display<'a, T>
where
    T: Print,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = Formatter::new(f, self.arena, Rc::clone(&self.state));
        Print::fmt(&self.data, &mut f)
    }
}

pub(crate) struct Sequence<'a, 'b, T> {
    data: &'a [T],
    separator: &'b str,
}

impl<'a, 'b, T> Sequence<'a, 'b, T> {
    pub(crate) fn new(data: &'a [T], separator: &'b str) -> Sequence<'a, 'b, T> {
        Sequence { data, separator }
    }
}

impl<'a, T> Print for Sequence<'_, '_, T>
where
    T: Print,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut first = true;
        for arg in self.data {
            if first {
                first = false;
            } else {
                f.write_str(self.separator)?;
            }
            Print::fmt(arg, f)?;
        }
        Ok(())
    }
}
