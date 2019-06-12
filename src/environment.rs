use std::sync::Arc;

use crate::{
    arena::{Arena, Index},
    data::{Function, Symbol, Variant},
    error::{Error, Fallible},
};

/// Execution context
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Environment {
    pub(crate) values: Vec<Variant>,
    pub(crate) names: Vec<Symbol>,
    heads: Vec<usize>, // TODO: call stack metadata
    packages: Vec<Arc<Package>>,
    fn_arena: Arena<Function>,
    arena: Arena<Variant>,
}

impl Environment {
    pub(crate) fn bind(&mut self, name: &str, value: Variant) {
        self.names.push(name.into());
        self.values.push(value);
    }

    pub(crate) fn lookup_name(&self, name: &str) -> Fallible<&Variant> {
        let i = self
            .names
            .iter()
            .position(|n| n == name)
            .ok_or_else(Error::name)?;
        Ok(&self.values[i])
    }

    pub(crate) fn lookup(&self, depth: usize, index: usize) -> Option<&Variant> {
        let i = self.heads.len().checked_sub(depth + 1).unwrap_or(0);
        let head = self.heads.get(i).copied().unwrap_or(0);
        self.values.get(head + index)
    }

    pub(crate) fn add_package(&mut self, pkg: Arc<Package>) {
        self.packages.push(pkg);
    }

    pub(crate) fn push(&mut self) {
        self.heads.push(self.values.len());
    }

    pub(crate) fn pop(&mut self) {
        if let Some(head) = self.heads.pop() {
            self.values.truncate(head);
        }
    }

    pub(crate) fn boxed(&mut self, value: Variant) -> Index<Variant> {
        self.arena.insert(value)
    }

    pub(crate) fn add_function(&mut self, f: Function) -> Index<Function> {
        self.fn_arena.insert(f)
    }

    pub(crate) fn get_function(&self, idx: Index<Function>) -> Option<&Function> {
        self.fn_arena.get(idx)
    }
}

#[cfg(not(test))]
impl ::std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.write_str("Environment")
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct Package {
    pub(crate) environment: Environment,
}
